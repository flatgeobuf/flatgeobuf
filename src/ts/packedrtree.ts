import Config from './config.js';
import Logger from './logger.js';

export const NODE_ITEM_LEN: number = 8 * 4 + 8;
// default branching factor of a node in the rtree
//
// actual value will be specified in the header but
// this can be useful for having reasonably sized guesses for fetch-sizes when
// streaming results
export const DEFAULT_NODE_SIZE = 16;

export interface Rect {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
}

export function calcTreeSize(numItems: number, nodeSize: number): number {
    nodeSize = Math.min(Math.max(+nodeSize, 2), 65535);
    let n = numItems;
    let numNodes = n;
    do {
        n = Math.ceil(n / nodeSize);
        numNodes += n;
    } while (n !== 1);
    return numNodes * NODE_ITEM_LEN;
}

/**
 * returns [levelOffset, numNodes] for each level
 */
export function generateLevelBounds(
    numItems: number,
    nodeSize: number,
): Array<[number, number]> {
    if (nodeSize < 2) throw new Error('Node size must be at least 2');
    if (numItems === 0)
        throw new Error('Number of items must be greater than 0');

    // number of nodes per level in bottom-up order
    let n = numItems;
    let numNodes = n;
    const levelNumNodes = [n];
    do {
        n = Math.ceil(n / nodeSize);
        numNodes += n;
        levelNumNodes.push(n);
    } while (n !== 1);

    // bounds per level in reversed storage order (top-down)
    const levelOffsets: Array<number> = [];
    n = numNodes;
    for (const size of levelNumNodes) {
        levelOffsets.push(n - size);
        n -= size;
    }
    const levelBounds: Array<[number, number]> = [];
    for (let i = 0; i < levelNumNodes.length; i++)
        levelBounds.push([levelOffsets[i], levelOffsets[i] + levelNumNodes[i]]);
    return levelBounds;
}

type ReadNodeFn = (treeOffset: number, size: number) => Promise<ArrayBuffer>;

/**
 * A feature found to be within the bounding box `rect`
 *
 *  (offset, index)
 *  `offset`: Byte offset in feature data section
 *  `index`: feature number
 *  `featureLength`: featureLength, except for the last element
 */
export type SearchResult = [number, number, number | null];

/**
 * Yield's a `SearchResult` for each feature within the bounds of `rect`.
 *
 * Every node in the FGB index tree has a bounding rect, all of the nodes children
 * are contained within that bounding rect. The leaf nodes of the tree represent
 * the features of the collection.
 *
 * As we traverse the tree, starting from the root, we'll need to read more data
 * from the index. When we don't already have this range data buffered locally,
 * an HTTP fetch is triggered. For performance, we merge adjacent and nearby
 * request ranges into a single request, reasoning that fetching a few extra
 * bytes is a good tradeoff if it means we can reduce the number of requests.
 */
export async function* streamSearch(
    numItems: number,
    nodeSize: number,
    rect: Rect,
    readNode: ReadNodeFn,
): AsyncGenerator<SearchResult, void, unknown> {
    class NodeRange {
        _level: number;
        nodes: [number, number];
        constructor(nodes: [number, number], level: number) {
            this._level = level;
            this.nodes = nodes;
        }

        level(): number {
            return this._level;
        }

        startNode(): number {
            return this.nodes[0];
        }

        endNode(): number {
            return this.nodes[1];
        }

        extendEndNodeToNewOffset(newOffset: number) {
            console.assert(newOffset > this.nodes[1]);
            this.nodes[1] = newOffset;
        }

        toString(): string {
            return `[NodeRange level: ${this._level}, nodes: ${this.nodes[0]}-${this.nodes[1]}]`;
        }
    }

    const { minX, minY, maxX, maxY } = rect;
    Logger.info(`tree items: ${numItems}, nodeSize: ${nodeSize}`);
    const levelBounds = generateLevelBounds(numItems, nodeSize);
    const leafNodesOffset = levelBounds[0][0];

    const rootNodeRange: NodeRange = (() => {
        const range: [number, number] = [0, 1];
        const level = levelBounds.length - 1;
        return new NodeRange(range, level);
    })();

    const queue: Array<NodeRange> = [rootNodeRange];

    Logger.debug(
        `starting stream search with queue: ${queue}, numItems: ${numItems}, nodeSize: ${nodeSize}, levelBounds: ${levelBounds}`,
    );

    while (queue.length != 0) {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const nodeRange = queue.shift()!;

        Logger.debug(`popped node: ${nodeRange}, queueLength: ${queue.length}`);

        const nodeIndex = nodeRange.startNode();
        const isLeafNode = nodeIndex >= leafNodesOffset;

        // find the end index of the node
        const [, levelBound] = levelBounds[nodeRange.level()];

        const end = Math.min(nodeRange.endNode() + nodeSize, levelBound);
        const length = end - nodeIndex;

        const buffer = await readNode(
            nodeIndex * NODE_ITEM_LEN,
            length * NODE_ITEM_LEN,
        );

        const dataView = new DataView(buffer);
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = (pos - nodeIndex) * 40;
            if (maxX < dataView.getFloat64(nodePos + 0, true)) continue; // maxX < nodeMinX
            if (maxY < dataView.getFloat64(nodePos + 8, true)) continue; // maxY < nodeMinY
            if (minX > dataView.getFloat64(nodePos + 16, true)) continue; // minX > nodeMaxX
            if (minY > dataView.getFloat64(nodePos + 24, true)) continue; // minY > nodeMaxY

            const offset = dataView.getBigUint64(nodePos + 32, true);

            if (isLeafNode) {
                const featureLength = (() => {
                    if (pos < numItems - 1) {
                        // Since features are tightly packed, we infer the
                        // length of _this_ feature by measuring to the _next_
                        // feature's start.
                        const nextPos = (pos - nodeIndex + 1) * 40;
                        const nextOffset = dataView.getBigUint64(
                            nextPos + 32,
                            true,
                        );
                        return nextOffset - offset;
                    } else {
                        // This is the last feature - there's no "next" feature
                        // to measure to, so we can't know it's length.
                        return null;
                    }
                })();

                // Logger.debug(`offset: ${offset}, pos: ${pos}, featureLength: ${featureLength}`);
                yield [
                    Number(offset),
                    pos - leafNodesOffset,
                    Number(featureLength),
                ];
                continue;
            }

            // request up to this many nodes if it means we can eliminate an
            // extra request
            const extraRequestThresholdNodes =
                Config.global.extraRequestThreshold() / NODE_ITEM_LEN;

            // Since we're traversing the tree by monotonically increasing byte
            // offset, the most recently enqueued node range will be the
            // nearest, and thus presents the best candidate for merging.
            const nearestNodeRange = queue[queue.length - 1];
            if (
                nearestNodeRange !== undefined &&
                nearestNodeRange.level() == nodeRange.level() - 1 &&
                offset < nearestNodeRange.endNode() + extraRequestThresholdNodes
            ) {
                Logger.debug(
                    `Merging "nodeRange" request into existing range: ${nearestNodeRange}, newOffset: ${nearestNodeRange.endNode()} -> ${offset}`,
                );
                nearestNodeRange.extendEndNodeToNewOffset(Number(offset));
                continue;
            }

            const newNodeRange: NodeRange = (() => {
                const level = nodeRange.level() - 1;
                const range: [number, number] = [
                    Number(offset),
                    Number(offset) + 1,
                ];
                return new NodeRange(range, level);
            })();

            // We're going to add a new node range - log the reason
            if (
                nearestNodeRange !== undefined &&
                nearestNodeRange.level() == newNodeRange.level()
            ) {
                Logger.info(
                    `Same level, but too far away. Pushing new request at offset: ${offset} rather than merging with distant ${nearestNodeRange}`,
                );
            } else {
                Logger.info(
                    `Pushing new level for ${newNodeRange} onto queue with nearestNodeRange: ${nearestNodeRange} since there's not already a range for this level.`,
                );
            }

            queue.push(newNodeRange);
        }
    }
}

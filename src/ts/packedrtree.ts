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

        const float64Array = new Float64Array(buffer);
        const uint32Array = new Uint32Array(buffer);
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = (pos - nodeIndex) * 5;
            if (maxX < float64Array[nodePos + 0]) continue; // maxX < nodeMinX
            if (maxY < float64Array[nodePos + 1]) continue; // maxY < nodeMinY
            if (minX > float64Array[nodePos + 2]) continue; // minX > nodeMaxX
            if (minY > float64Array[nodePos + 3]) continue; // minY > nodeMaxY

            const low32Offset = uint32Array[(nodePos << 1) + 8];
            const high32Offset = uint32Array[(nodePos << 1) + 9];
            const offset = readUint52(high32Offset, low32Offset);

            if (isLeafNode) {
                const featureLength = (() => {
                    if (pos < numItems - 1) {
                        // Since features are tightly packed, we infer the
                        // length of _this_ feature by measuring to the _next_
                        // feature's start.
                        const nextPos = (pos - nodeIndex + 1) * 5;
                        const low32Offset = uint32Array[(nextPos << 1) + 8];
                        const high32Offset = uint32Array[(nextPos << 1) + 9];
                        const nextOffset = readUint52(
                            high32Offset,
                            low32Offset,
                        );

                        return nextOffset - offset;
                    } else {
                        // This is the last feature - there's no "next" feature
                        // to measure to, so we can't know it's length.
                        return null;
                    }
                })();

                // Logger.debug(`offset: ${offset}, pos: ${pos}, featureLength: ${featureLength}`);
                yield [offset, pos - leafNodesOffset, featureLength];
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
                nearestNodeRange.extendEndNodeToNewOffset(offset);
                continue;
            }

            const newNodeRange: NodeRange = (() => {
                const level = nodeRange.level() - 1;
                const range: [number, number] = [offset, offset + 1];
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

/**
 * Returns a 64-bit uint value by combining it's decomposed lower and higher
 * 32-bit halves. Though because JS `number` is a floating point, it cannot
 * accurately represent an int beyond 52 bits.
 *
 * In practice, "52-bits ought to be enough for anybody", or at least into the
 * pebibytes.
 *
 * Note: `BigInt` does exist to hold larger numbers, but we'd have to adapt a
 * lot of code to support using it.
 */
function readUint52(high32Bits: number, low32Bits: number) {
    // javascript integers can only be 52 bits, verify the top 12 bits
    // are unused.
    if ((high32Bits & 0xfff00000) != 0) {
        throw Error('integer is too large to be safely represented');
    }

    // Note: we multiply by 2**32 because bitshift operations wrap at 32, so `high32Bits << 32` would be a NOOP.
    const result = low32Bits + high32Bits * 2 ** 32;

    return result;
}

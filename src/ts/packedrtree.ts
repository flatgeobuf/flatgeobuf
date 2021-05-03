const NODE_ITEM_LEN: number = 8 * 4 + 8

export interface Rect {
    minX: number
    minY: number
    maxX: number
    maxY: number
}

export function calcTreeSize(numItems: number, nodeSize: number): number {
    nodeSize = Math.min(Math.max(+nodeSize, 2), 65535)
    let n = numItems
    let numNodes = n
    do {
        n = Math.ceil(n / nodeSize)
        numNodes += n
    } while (n !== 1)
    return numNodes * NODE_ITEM_LEN
}

function generateLevelBounds(numItems: number, nodeSize: number) {
    if (nodeSize < 2)
        throw new Error('Node size must be at least 2')
    if (numItems === 0)
        throw new Error('Number of items must be greater than 0')

    // number of nodes per level in bottom-up order
    let n = numItems
    let numNodes = n
    const levelNumNodes = [n]
    do {
        n = Math.ceil(n / nodeSize)
        numNodes += n
        levelNumNodes.push(n)
    } while (n !== 1)

    // bounds per level in reversed storage order (top-down)
    const levelOffsets = []
    n = numNodes
    for (const size of levelNumNodes) {
        levelOffsets.push(n - size)
        n -= size
    }
    levelOffsets.reverse()
    levelNumNodes.reverse()
    const levelBounds = []
    for (let i = 0; i < levelNumNodes.length; i++)
        levelBounds.push([levelOffsets[i], levelOffsets[i] + levelNumNodes[i]])
    levelBounds.reverse()
    return levelBounds
}

type ReadNodeFn = (treeOffset: number, size: number) => Promise<ArrayBuffer>

export async function* streamSearch(
    numItems: number,
    nodeSize: number,
    rect: Rect,
    readNode: ReadNodeFn): AsyncGenerator<number[], void, unknown>
{
    const { minX, minY, maxX, maxY } = rect
    const levelBounds = generateLevelBounds(numItems, nodeSize)
    const [[leafNodesOffset,numNodes]] = levelBounds
    const queue: any[] = []
    queue.push([0, levelBounds.length - 1])
    while (queue.length !== 0) {
        const [nodeIndex, level] = queue.pop()
        const isLeafNode = nodeIndex >= numNodes - numItems
        // find the end index of the node
        const [,levelBound] = levelBounds[level]
        const end = Math.min(nodeIndex + nodeSize, levelBound)
        const length = end - nodeIndex
        const buffer = await readNode(nodeIndex * NODE_ITEM_LEN, length * NODE_ITEM_LEN)
        const float64Array = new Float64Array(buffer)
        const uint32Array = new Uint32Array(buffer)
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = (pos - nodeIndex) * 5
            if (maxX < float64Array[nodePos + 0]) continue // maxX < nodeMinX
            if (maxY < float64Array[nodePos + 1]) continue // maxY < nodeMinY
            if (minX > float64Array[nodePos + 2]) continue // minX > nodeMaxX
            if (minY > float64Array[nodePos + 3]) continue // minY > nodeMaxY

            const low32Offset = uint32Array[(nodePos << 1) + 8]
            const high32Offset = uint32Array[(nodePos << 1) + 9]
            const offset = readUint52(high32Offset, low32Offset);

            if (isLeafNode)
                yield [offset, pos - leafNodesOffset]
            else
                queue.push([offset, level - 1])
        }
        // order queue to traverse sequential
        queue.sort((a, b) => b[0] - a[0])
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
    if ((high32Bits & 0xfff00000) != 0)  {
        throw Error("integer is too large to be safely represented");
    }

    // Note: we multiply by 2**32 because bitshift operations wrap at 32, so `high32Bits << 32` would be a NOOP.
    const result = low32Bits + (high32Bits * 2**32);

    return result;
}

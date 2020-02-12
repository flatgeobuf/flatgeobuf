const NODE_ITEM_LEN: number = 8 * 4 + 8

export interface Rect {
    minX: number
    minY: number
    maxX: number
    maxY: number
}

export function calcTreeSize(numItems: number, nodeSize: number) {
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
    for (let size of levelNumNodes) {
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

interface IReadNode {
    (treeOffset: number, size: number): Promise<ArrayBuffer>
}

type ReadNodeFn = (treeOffset: number, size: number) => Promise<ArrayBuffer>

export async function streamSearch(numItems: number, nodeSize: number, rect: Rect, readNode: ReadNodeFn)
{
    const { minX, minY, maxX, maxY } = rect
    const levelBounds = generateLevelBounds(numItems, nodeSize)
    const [[,numNodes]] = levelBounds
    const queue = []
    const results = []
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
        for (let i = 0; i < length * 5; i += 5) {
            if (maxX < float64Array[i + 0]) continue // maxX < nodeMinX
            if (maxY < float64Array[i + 1]) continue // maxY < nodeMinY
            if (minX > float64Array[i + 2]) continue // minX > nodeMaxX
            if (minY > float64Array[i + 3]) continue // minY > nodeMaxY
            const offset = uint32Array[(i << 1) + 8]
            if (isLeafNode)
                results.push(offset)
            else
                queue.push([offset, level - 1])
        }
        // order queue to traverse sequential
        queue.sort((a, b) => b[0] - a[0])
    }
    return results
}
const NODE_ITEM_LEN: number = 8 * 4 + 8

export interface Rect {
    minX: number
    minY: number
    maxX: number
    maxY: number
}

function intersects(a: Rect, b: Rect) {
    if (a.maxX < b.minX) return false
    if (a.maxY < b.minY) return false
    if (a.minX > b.maxX) return false
    if (a.minY > b.maxY) return false
    return true
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

export async function streamSearch(numItems: number, nodeSize: number, rect: Rect, readNode)
{
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
        const nodeItems = []
        for (let i = 0; i < length * 5; i += 5) {
            const minX = float64Array[i + 0]
            const minY = float64Array[i + 1]
            const maxX = float64Array[i + 2]
            const maxY = float64Array[i + 3]
            const offset = uint32Array[(i << 1) + 8]
            nodeItems.push({ minX, minY, maxX, maxY, offset })
        }
        // search through child nodes
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = pos - nodeIndex
            const nodeItem = nodeItems[nodePos]
            if (!intersects(rect, nodeItem))
                continue
            if (isLeafNode)
                results.push(nodeItem.offset)
            else
                queue.push([nodeItem.offset, level - 1])
        }
        // order queue to traverse sequential
        queue.sort((a, b) => b[0] - a[0])
    }
    return results
}
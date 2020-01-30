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
    const levelOffsets = [];
    n = numNodes;
    for (let size of levelNumNodes) {
        levelOffsets.push(n - size)
        n -= size
    }
    levelOffsets.reverse()
    levelNumNodes.reverse()
    const levelBounds = []
    for (let i = 0; i < levelNumNodes.length; i++)
        levelBounds.push([levelOffsets[i], levelOffsets[i] + levelNumNodes[i]]);
    levelBounds.reverse()
    return levelBounds
}

export async function streamSearch(numItems: number, nodeSize: number, rect: Rect, readNode)
{
    const levelBounds = generateLevelBounds(numItems, nodeSize)
    const numNodes = levelBounds[0][1]
    const queue = []
    const results = []
    queue.push([0, levelBounds.length - 1])
    while (queue.length !== 0) {
        let [nodeIndex, level] = queue.pop()
        const isLeafNode = nodeIndex >= numNodes - numItems;
        // find the end index of the node
        const end = Math.min(nodeIndex + nodeSize, levelBounds[level][1])
        const length = end - nodeIndex
        const dataView = new DataView((await readNode(nodeIndex * NODE_ITEM_LEN, length * NODE_ITEM_LEN)).buffer)
        const nodeItems = []
        for (let i = 0; i < length; i++) {
            const minX = dataView.getFloat64(i * NODE_ITEM_LEN + 0, true)
            const minY = dataView.getFloat64(i * NODE_ITEM_LEN + 8, true)
            const maxX = dataView.getFloat64(i * NODE_ITEM_LEN + 16, true)
            const maxY = dataView.getFloat64(i * NODE_ITEM_LEN + 24, true)
            const offset = Number(dataView.getBigUint64(i * NODE_ITEM_LEN + 32, true))
            nodeItems.push({ minX, minY, maxX, maxY, offset })
        }
        // search through child nodes
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = pos - nodeIndex
            const nodeItem = nodeItems[nodePos]
            if (!intersects(rect, nodeItem))
                continue
            if (isLeafNode)
                results.push([nodeItem, pos - 1])
            else
                queue.push([nodeItem.offset, level - 1])
        }
        // order queue to traverse sequential
        queue.sort((a, b) => b[0] - a[0])
    }
    return results
}
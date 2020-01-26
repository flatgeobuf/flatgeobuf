const NODE_RECT_LEN: number = 8 * 4
const NODE_INDEX_LEN: number = 4

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
    const numNonLeafNodes = numNodes - numItems
    const minAlign = numNonLeafNodes % 2
    return numNodes * NODE_RECT_LEN + (numNonLeafNodes + minAlign) * NODE_INDEX_LEN
}

function generateLevelBounds(numItems: number, nodeSize: number) {
    if (nodeSize < 2)
        throw new Error('Node size must be at least 2')
    if (numItems === 0)
        throw new Error('Number of items must be greater than 0')
    let n = numItems
    let numNodes = n
    const levelBounds = [n]
    do {
        n = Math.ceil(n / nodeSize)
        numNodes += n
        levelBounds.push(numNodes)
    } while (n !== 1)
    return levelBounds
}

export async function streamSearch(numItems: number, nodeSize: number, rect: Rect, readNode)
{
    const levelBounds = generateLevelBounds(numItems, nodeSize)
    const numNodes = levelBounds[levelBounds.length - 1]
    const queue = []
    const results = []
    queue.push(numNodes - 1)
    queue.push(levelBounds.length - 1)
    while(queue.length !== 0) {
        const nodeIndex = queue[queue.length - 2]
        const isLeafNode = nodeIndex < numItems
        const level = queue[queue.length - 1]
        queue.pop()
        queue.pop()
        // find the end index of the node
        const end = Math.min(nodeIndex + nodeSize, levelBounds[level])
        const length = end - nodeIndex
        const nodeIndices = []
        if (!isLeafNode) {
            const offset = numNodes * 32 + (nodeIndex - numItems) * 4
            const dataView = new DataView((await readNode(offset, length * 4)).buffer)
            for (let i = 0; i < length; i++) {
                const index = dataView.getUint32(i * 4, true)
                nodeIndices.push(index)
            }
        }
        const offset = nodeIndex * 32
        const dataView = new DataView((await readNode(offset, length * 32)).buffer)
        const nodeRects = []
        for (let i = 0; i < length; i++) {
            const minX = dataView.getFloat64(i * 32 + 0, true)
            const minY = dataView.getFloat64(i * 32 + 8, true)
            const maxX = dataView.getFloat64(i * 32 + 16, true)
            const maxY = dataView.getFloat64(i * 32 + 24, true)
            nodeRects.push({ minX, minY, maxX, maxY })
        }

        // search through child nodes
        for (let pos = nodeIndex; pos < end; pos++) {
            const nodePos = pos - nodeIndex;
            if (!intersects(rect, nodeRects[nodePos]))
                continue
            if (isLeafNode) {
                results.push(pos) // leaf item
            } else {
                queue.push(nodeIndices[nodePos]) // node; add it to the search queue
                queue.push(level - 1)
            }
        }
    }
    return results
}
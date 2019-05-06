const NODE_RECT_LEN: number = 8 * 4
const NODE_INDEX_LEN: number = 4

export function size(numItems: number, nodeSize: number) {
    nodeSize = Math.min(Math.max(+nodeSize, 2), 65535)
    let n = numItems
    let numNodes = n
    do {
        n = Math.ceil(n / nodeSize)
        numNodes += n
    } while (n !== 1)
    return numNodes * NODE_RECT_LEN + (numNodes - numItems) * NODE_INDEX_LEN
}
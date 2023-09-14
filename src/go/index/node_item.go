package index

import "math"

// NodeItem represents a node in our RTree.
type NodeItem struct {
	minX, minY, maxX, maxY float64
	offset                 uint64
}

// NewNodeITem creates a new NodeItem.
func NewNodeItem(offset uint64) NodeItem {
	return NodeItem{
		math.Inf(1),
		math.Inf(1),
		math.Inf(-1),
		math.Inf(-1),
		offset,
	}
}

// NewNodeItemWithCoordinates creates a new NodeItem with the given coordinates.
func NewNodeItemWithCoordinates(offset uint64, minX, minY, maxX, maxY float64) NodeItem {
	return NodeItem{
		minX,
		minY,
		maxX,
		maxY,
		offset,
	}
}

// Width returns the width of the NodeItem.
func (n NodeItem) Width() float64 {
	return n.maxX - n.minX
}

// Height returns the height of the NodeItem.
func (n NodeItem) Height() float64 {
	return n.maxY - n.minY
}

// Sum expands this NodeItem by the given NodeItem.
func (n *NodeItem) Sum(nodeItem NodeItem) {
	n.Expand(nodeItem)
}

// Intersects returns true if this NodeItem intersects with the given NodeItem.
func (n NodeItem) Intersects(nodeItem *NodeItem) bool {
	return n.minX <= nodeItem.maxX &&
		n.maxX >= nodeItem.minX &&
		n.minY <= nodeItem.maxY &&
		n.maxY >= nodeItem.minY
}

// Expand expands this NodeItem by the given NodeItem.
func (n *NodeItem) Expand(nodeItem NodeItem) {
	n.minX = math.Min(n.minX, nodeItem.minX)
	n.minY = math.Min(n.minY, nodeItem.minY)
	n.maxX = math.Max(n.maxX, nodeItem.maxX)
	n.maxY = math.Max(n.maxY, nodeItem.maxY)
}

// ToSlice returns the NodeItem as a slice oif its coordinates (minX, minY,
// maxX, maxY).
func (n NodeItem) ToSlice() []float64 {
	return []float64{n.minX, n.minY, n.maxX, n.maxY}
}

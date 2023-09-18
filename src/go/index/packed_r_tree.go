package index

import (
	"fmt"
	"io"
	"math"
	"sort"
	"unsafe"
)

// PackedRTree is a packed RTree implementation using Hilbert curves.
type PackedRTree struct {
	extent      NodeItem
	nodeItems   []NodeItem
	numItems    uint64
	numNodes    uint64
	nodeSize    uint16
	levelBounds []LevelBound
}

// NewPackedRTreeWithItems creates a new PackedRTree from items with the given
// extent and node size.
func NewPackedRTreeWithItems(items []Item, extent NodeItem,
	nodeSize uint16) *PackedRTree {
	r := &PackedRTree{
		extent:   extent,
		numItems: uint64(len(items)),
	}

	r.init(nodeSize)

	for i := 0; i < int(r.numItems); i++ {
		r.nodeItems[r.numNodes-r.numItems+uint64(i)] = items[i].NodeItem()
	}

	r.generateNodes()

	return r
}

// NewPackedRTreeWithNodeItems creates a new PackedRTree from nodeItems with the
// given extent and node size.
func NewPackedRTreeWithNodeItems(nodeItems []NodeItem, extent NodeItem,
	nodeSize uint16) *PackedRTree {
	r := &PackedRTree{
		extent:   extent,
		numItems: uint64(len(nodeItems)),
	}

	r.init(nodeSize)

	for i := 0; i < int(r.numItems); i++ {
		r.nodeItems[r.numNodes-r.numItems+uint64(i)] = nodeItems[i]
	}

	r.generateNodes()

	return r
}

// NewPackedRTreeFromData creates a new PackedRTree from data with the given
// number of items and node size.
func NewPackedRTreeFromData(data []byte, numItems uint64,
	nodeSize uint16, copyData bool) *PackedRTree {
	r := &PackedRTree{
		extent:   NewNodeItem(0),
		numItems: numItems,
	}

	r.init(nodeSize)
	r.fromData(data, copyData)

	return r
}

// Search returns all items in the RTree that intersect with the given bounding
// box.
func (r *PackedRTree) Search(
	minX, minY, maxX, maxY float64) []SearchResultItem {
	leafNodesOffset := r.levelBounds[0].Start
	n := NodeItem{
		minX:   minX,
		minY:   minY,
		maxX:   maxX,
		maxY:   maxY,
		offset: 0,
	}

	var results []SearchResultItem

	// In place pair definition for convenience.
	type pair struct {
		first  uint64
		second uint64
	}

	// Use a slice as a queue. This is simple to implement but items removed
	// from the slice are not deallocated so if the queue grows large a
	// considerable amount of memory might get leaked (it will be, of course,
	// garbage-collected after this method returns).
	queue := make([]pair, 1)
	queue[0] = pair{0, uint64(len(r.levelBounds) - 1)}
	for len(queue) != 0 {
		next := queue[0]
		nodeIndex := next.first
		level := next.second
		queue = queue[1:]
		isLeafNode := nodeIndex >= r.numNodes-r.numItems
		end := uint64(math.Min(float64(nodeIndex)+float64(r.nodeSize),
			float64(r.levelBounds[level].End)))
		for pos := nodeIndex; pos < end; pos++ {
			nodeItem := r.nodeItems[pos]
			if !nodeItem.Intersects(&n) {
				continue
			}
			if isLeafNode {
				results = append(results, SearchResultItem{
					Offset: nodeItem.offset,
					Index:  pos - leafNodesOffset,
				})
			} else {
				queue = append(queue, pair{nodeItem.offset, level - 1})
			}
		}
	}

	sort.Sort(ByOffset(results))

	return results
}

func (r *PackedRTree) Size() uint64 {
	return r.numNodes * uint64(unsafe.Sizeof(NodeItem{}))
}

// Write writes the RTree to the given writer. The format is suitable for use
// with NewPackedRTreeFromData.
func (r *PackedRTree) Write(w io.Writer) (int, error) {
	// Fast conversion from  []NodeItems to []byte.
	data := (*(*[1 << 31]byte)(unsafe.Pointer(
		&r.nodeItems[0])))[:r.Size()]

	return w.Write(data)
}

func (r *PackedRTree) init(nodeSize uint16) error {
	if nodeSize < 2 {
		return fmt.Errorf("nodeSize must be >= 2")
	}

	if r.numItems == 0 {
		return fmt.Errorf("cannot create empty tree")
	}

	var err error

	r.nodeSize = nodeSize
	r.levelBounds, err = r.generateLevelBounds(r.numItems, r.nodeSize)
	if err != nil {
		return err
	}
	r.numNodes = r.levelBounds[0].End
	r.nodeItems = make([]NodeItem, r.numNodes)

	return nil
}

func (r *PackedRTree) generateLevelBounds(numItems uint64, nodeSize uint16) ([]LevelBound, error) {
	if nodeSize < 2 {
		return nil, fmt.Errorf("nodeSize must be >= 2")
	}

	if numItems == 0 {
		return nil, fmt.Errorf("cannot create empty tree")
	}

	levelNumNodes := make([]uint64, 0)
	n := numItems
	numNodes := n
	levelNumNodes = append(levelNumNodes, n)
	for ok := true; ok; ok = (n != 1) {
		n = (n + uint64(nodeSize) - 1) / uint64(nodeSize)
		numNodes += n
		levelNumNodes = append(levelNumNodes, n)
	}

	levelOffsets := make([]uint64, 0)
	n = numNodes
	for _, size := range levelNumNodes {
		levelOffsets = append(levelOffsets, n-size)
		n -= size
	}

	levelBounds := make([]LevelBound, 0, len(levelNumNodes))
	for i := 0; i < len(levelNumNodes); i++ {
		levelBounds = append(
			levelBounds, LevelBound{
				levelOffsets[i],
				levelOffsets[i] + levelNumNodes[i],
			})
	}

	return levelBounds, nil
}

func (r *PackedRTree) generateNodes() {
	for i := 0; i < len(r.levelBounds)-1; i++ {
		pos := r.levelBounds[i].Start
		end := r.levelBounds[i].End
		newPos := r.levelBounds[i+1].Start
		for pos < end {
			node := NewNodeItem(pos)
			for j := 0; j < int(r.nodeSize) && pos < end; j++ {
				node.Expand(r.nodeItems[pos])
				pos++
			}
			r.nodeItems[newPos] = node
			newPos++
		}
	}
}

func (r *PackedRTree) fromData(data []byte, copyData bool) {
	var finalData []byte
	if copyData {
		finalData = make([]byte, r.Size())
		copy(finalData, data[:r.Size()])
	} else {
		finalData = data
	}

	// Fast conversion from []byte to []NodeItem.
	r.nodeItems = (*[1 << 31]NodeItem)(unsafe.Pointer(&finalData[0]))[:r.numNodes]

	for i := 0; i < int(r.numNodes); i++ {
		r.extent.Expand(r.nodeItems[i])
	}
}

package index

import (
	"bytes"
	"math/rand"
	"testing"
	"unsafe"
)

func TestPackedRTree(t *testing.T) {
	nodes := []NodeItem{
		{0, 0, 1, 1, 0},
		{2, 2, 3, 3, 0},
	}

	extent := CalcExtentForNodeItems(nodes)

	intersectsNode := &NodeItem{0, 0, 1, 1, 0}
	if !nodes[0].Intersects(intersectsNode) {
		t.Errorf("Intersects failed")
	}
	intersectsNode = &NodeItem{2, 2, 3, 4, 0}
	if !nodes[1].Intersects(intersectsNode) {
		t.Errorf("Intersects failed")
	}

	HilbertSortNodeItems(nodes)

	offset := 0

	for _, node := range nodes {
		node.offset = uint64(offset) + uint64(unsafe.Sizeof(NodeItem{}))
		offset += int(unsafe.Sizeof(NodeItem{}))
	}

	intersectsNode = &NodeItem{0, 0, 1, 1, 0}
	if !nodes[1].Intersects(intersectsNode) {
		t.Errorf("Intersects failed")
	}

	intersectsNode = &NodeItem{2, 2, 3, 4, 0}
	if !nodes[0].Intersects(intersectsNode) {
		t.Errorf("Intersects failed")
	}

	tree := NewPackedRTreeWithNodeItems(nodes, extent, 2)

	results := tree.Search(0, 0, 1, 1)
	if len(results) != 1 {
		t.Errorf("Search failed")
	}
	intersectsNode = &NodeItem{0, 0, 1, 1, 0}
	if !nodes[results[0].Index].Intersects(intersectsNode) {
		t.Errorf("Search failed")
	}
}

func TestRoundTrip(t *testing.T) {
	nodeItems := make([]NodeItem, 100000, 100000)

	xMean := (466379 + 708929) / 2.0
	xStdDev := (708929 - 466379) / 6.0
	yMean := (6096801 + 6322352) / 2.0
	yStdDev := (6322352 - 6096801) / 6.0

	for i := 0; i < len(nodeItems); i++ {
		x := rand.NormFloat64()*xStdDev + xMean
		y := rand.NormFloat64()*yStdDev + yMean
		nodeItems[i].minX = x
		nodeItems[i].minY = y
		nodeItems[i].maxX = x
		nodeItems[i].maxY = y
		nodeItems[i].offset = uint64(i)
	}

	extent := CalcExtentForNodeItems(nodeItems)

	HilbertSortNodeItems(nodeItems)

	tree := NewPackedRTreeWithNodeItems(nodeItems, extent, 16)

	sris := tree.Search(690407, 6063692, 811682, 6176467)

	for _, sri := range sris {
		if !nodeItems[sri.Index].Intersects(&NodeItem{690407, 6063692, 811682, 6176467, 0}) {
			t.Errorf("Search failed")
		}
	}

	var b bytes.Buffer
	_, err := tree.Write(&b)
	if err != nil {
		t.Errorf("Write failed")
	}

	tree2 := NewPackedRTreeFromData(b.Bytes(), uint64(len(nodeItems)), 16, false)
	sris2 := tree2.Search(690407, 6063692, 811682, 6176467)

	for i, sri2 := range sris2 {
		if sri2.Index != sris[i].Index {
			t.Errorf("Search failed")
		}
	}

	// Test with data copy.
	tree3 := NewPackedRTreeFromData(b.Bytes(), uint64(len(nodeItems)), 16, true)
	sris3 := tree3.Search(690407, 6063692, 811682, 6176467)

	for i, sri3 := range sris3 {
		if sri3.Index != sris[i].Index {
			t.Errorf("Search failed")
		}
	}
}

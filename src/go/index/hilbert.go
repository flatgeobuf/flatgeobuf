package index

import (
	"math"
	"sort"
)

const (
	// HilbertMax is the maximum value of a Hilbert curve coordinate.
	HilbertMax = uint32((1 << 16) - 1)
)

// Hilbert implements a Hilbert curve mapping from 2D to 1D. More info:
//
// https://en.wikipedia.org/wiki/Hilbert_curve
//
// The specific implementation below is based on the public domain code here:
//
// https://github.com/rawrunprotected/hilbert_curves
func Hilbert(x, y uint32) uint32 {
	a := x ^ y
	b := 0xFFFF ^ a
	c := 0xFFFF ^ (x | y)
	d := x & (y ^ 0xFFFF)

	A := a | (b >> 1)
	B := (a >> 1) ^ a
	C := ((c >> 1) ^ (b & (d >> 1))) ^ c
	D := ((a & (c >> 1)) ^ (d >> 1)) ^ d

	a = A
	b = B
	c = C
	d = D
	A = ((a & (a >> 2)) ^ (b & (b >> 2)))
	B = ((a & (b >> 2)) ^ (b & ((a ^ b) >> 2)))
	C ^= ((a & (c >> 2)) ^ (b & (d >> 2)))
	D ^= ((b & (c >> 2)) ^ ((a ^ b) & (d >> 2)))

	a = A
	b = B
	c = C
	d = D
	A = ((a & (a >> 4)) ^ (b & (b >> 4)))
	B = ((a & (b >> 4)) ^ (b & ((a ^ b) >> 4)))
	C ^= ((a & (c >> 4)) ^ (b & (d >> 4)))
	D ^= ((b & (c >> 4)) ^ ((a ^ b) & (d >> 4)))

	a = A
	b = B
	c = C
	d = D
	C ^= ((a & (c >> 8)) ^ (b & (d >> 8)))
	D ^= ((b & (c >> 8)) ^ ((a ^ b) & (d >> 8)))

	a = C ^ (C >> 1)
	b = D ^ (D >> 1)

	i0 := x ^ y
	i1 := b | (0xFFFF ^ (i0 | a))

	i0 = (i0 | (i0 << 8)) & 0x00FF00FF
	i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F
	i0 = (i0 | (i0 << 2)) & 0x33333333
	i0 = (i0 | (i0 << 1)) & 0x55555555

	i1 = (i1 | (i1 << 8)) & 0x00FF00FF
	i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F
	i1 = (i1 | (i1 << 2)) & 0x33333333
	i1 = (i1 | (i1 << 1)) & 0x55555555

	value := ((i1 << 1) | i0)

	return value
}

// HilbertForNodeItem calculates the Hilbert curve value for a NodeItem.
func HilbertForNodeItem(nodeItem NodeItem, hilbertMax uint32,
	minX, minY, width, height float64) uint32 {
	var x, y uint32 = 0, 0
	if width != 0 {
		x = uint32(math.Floor(float64(hilbertMax) * ((nodeItem.minX+nodeItem.maxX)/2 - minX) / width))
	}
	if height != 0 {
		y = uint32(math.Floor(float64(hilbertMax) * ((nodeItem.minY+nodeItem.maxY)/2 - minY) / height))
	}

	return Hilbert(x, y)
}

// HilbertSortItems sorts items by Hilbert curve value.
func HilbertSortItems(items []Item) {
	extent := CalcExtentForItems(items)
	minX := extent.minX
	minY := extent.minY
	width := extent.maxX - extent.minX
	height := extent.maxY - extent.minY

	sort.Slice(items, func(i, j int) bool {
		ha := HilbertForNodeItem(items[i].NodeItem(), HilbertMax, minX, minY, width, height)
		hb := HilbertForNodeItem(items[j].NodeItem(), HilbertMax, minX, minY, width, height)
		return ha > hb
	})
}

// HilbertSortNodeItems sorts nodeItems by Hilbert curve value.
func HilbertSortNodeItems(nodeItems []NodeItem) {
	extent := CalcExtentForNodeItems(nodeItems)
	minX := extent.minX
	minY := extent.minY
	width := extent.maxX - extent.minX
	height := extent.maxY - extent.minY

	sort.Slice(nodeItems, func(i, j int) bool {
		ha := HilbertForNodeItem(nodeItems[i], HilbertMax, minX, minY, width, height)
		hb := HilbertForNodeItem(nodeItems[j], HilbertMax, minX, minY, width, height)
		return ha > hb
	})
}

func CalcExtentForItems(items []Item) NodeItem {
	n := NewNodeItem(0)
	for _, item := range items {
		n.Expand(item.NodeItem())
	}

	return n
}

func CalcExtentForNodeItems(nodeItems []NodeItem) NodeItem {
	n := NewNodeItem(0)
	for _, nodeItem := range nodeItems {
		n.Expand(nodeItem)
	}

	return n
}

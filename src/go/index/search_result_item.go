package index

// SearchResultItem represents a search result item. It includes the
// offset and index of the item in the Flatgeobuf file.
type SearchResultItem struct {
	Offset uint64
	Index  uint64
}

// Implement the Sort interface for SearchResultItems.
type ByOffset []SearchResultItem

func (s ByOffset) Len() int {
	return len(s)
}

func (s ByOffset) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

func (s ByOffset) Less(i, j int) bool {
	return s[i].Offset < s[j].Offset
}

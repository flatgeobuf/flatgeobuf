package index

// Item represents an item in our RTree. It is just a wrapper around NodeItem.
//
// TODO(bga): We most likelly can get hid of this.
type Item interface {
	NodeItem() NodeItem
}

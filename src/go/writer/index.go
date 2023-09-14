package writer

import (
	"io"

	"github.com/flatgeobuf/flatgeobuf/src/go/index"
)

type Index struct {
	p *index.PackedRTree
}

func NewIndex(p *index.PackedRTree) *Index {
	return &Index{
		p: p,
	}
}

func (i *Index) Write(w io.Writer) (int, error) {
	return i.p.Write(w)
}

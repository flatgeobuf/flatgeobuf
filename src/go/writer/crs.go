package writer

import (
	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	flatbuffers "github.com/google/flatbuffers/go"
)

// Crs is a builder for flatgeobuf.Crs, which represents the coordinate reference
// system associated with the data stored in the flatgeobuf. This is not
// required to be set at all.
type Crs struct {
	builder *flatbuffers.Builder

	org         string
	code        int32
	name        string
	description string
	codeString  string
}

func NewCrs(builder *flatbuffers.Builder) *Crs {
	return &Crs{
		builder: builder,
	}
}

func (c *Crs) SetOrg(org string) *Crs {
	c.org = org
	return c
}

func (c *Crs) SetCode(code int32) *Crs {
	c.code = code
	return c
}

func (c *Crs) SetName(name string) *Crs {
	c.name = name
	return c
}

func (c *Crs) SetDescription(description string) *Crs {
	c.description = description
	return c
}

func (c *Crs) SetCodeString(codeString string) *Crs {
	c.codeString = codeString
	return c
}

func (c *Crs) Build() flatbuffers.UOffsetT {
	if c == nil || c.builder == nil {
		return 0
	}

	orgOffset := maybeCreateString(c.builder, c.org)
	nameOffset := maybeCreateString(c.builder, c.name)
	descriptionOffset := maybeCreateString(c.builder, c.description)
	codeStringOffset := maybeCreateString(c.builder, c.codeString)

	flattypes.CrsStart(c.builder)
	flattypes.CrsAddOrg(c.builder, orgOffset)
	flattypes.CrsAddCode(c.builder, c.code)
	flattypes.CrsAddName(c.builder, nameOffset)
	flattypes.CrsAddDescription(c.builder, descriptionOffset)
	flattypes.CrsAddCodeString(c.builder, codeStringOffset)

	return flattypes.CrsEnd(c.builder)
}

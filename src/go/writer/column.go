package writer

import (
	flatbuffers "github.com/google/flatbuffers/go"

	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
)

type Column struct {
	builder *flatbuffers.Builder

	name        string
	typ         flattypes.ColumnType
	title       string
	description string
	width       int
	precision   int
	scale       int
	nullable    bool
	unique      bool
	primaryKey  bool
	metadata    string
}

func NewColumn(builder *flatbuffers.Builder) *Column {
	return &Column{
		builder: builder,
	}
}

func (c *Column) SetName(name string) *Column {
	c.name = name
	return c
}

func (c *Column) SetType(typ flattypes.ColumnType) *Column {
	c.typ = typ
	return c
}

func (c *Column) SetTitle(title string) *Column {
	c.title = title
	return c
}

func (c *Column) SetDescription(description string) *Column {
	c.description = description
	return c
}

func (c *Column) SetWidth(width int) *Column {
	c.width = width
	return c
}

func (c *Column) SetPrecision(precision int) *Column {
	c.precision = precision
	return c
}

func (c *Column) SetScale(scale int) *Column {
	c.scale = scale
	return c
}

func (c *Column) SetNullable(nullable bool) *Column {
	c.nullable = nullable
	return c
}

func (c *Column) SetUnique(unique bool) *Column {
	c.unique = unique
	return c
}

func (c *Column) SetPrimaryKey(primaryKey bool) *Column {
	c.primaryKey = primaryKey
	return c
}

func (c *Column) SetMetadata(metadata string) *Column {
	c.metadata = metadata
	return c
}

func (c *Column) Build() flatbuffers.UOffsetT {
	if c.builder == nil {
		return 0
	}

	nameOffset := maybeCreateString(c.builder, c.name)
	titleOffset := maybeCreateString(c.builder, c.title)
	descriptionOffset := maybeCreateString(c.builder, c.description)
	metadataOffset := maybeCreateString(c.builder, c.metadata)

	flattypes.ColumnStart(c.builder)

	flattypes.ColumnAddName(c.builder, nameOffset)
	flattypes.ColumnAddType(c.builder, c.typ)
	flattypes.ColumnAddTitle(c.builder, titleOffset)
	flattypes.ColumnAddDescription(c.builder, descriptionOffset)
	flattypes.ColumnAddWidth(c.builder, int32(c.width))
	flattypes.ColumnAddPrecision(c.builder, int32(c.precision))
	flattypes.ColumnAddScale(c.builder, int32(c.scale))
	flattypes.ColumnAddNullable(c.builder, c.nullable)
	flattypes.ColumnAddUnique(c.builder, c.unique)
	flattypes.ColumnAddPrimaryKey(c.builder, c.primaryKey)
	flattypes.ColumnAddMetadata(c.builder, metadataOffset)

	return flattypes.ColumnEnd(c.builder)
}

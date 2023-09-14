package writer

import (
	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	flatbuffers "github.com/google/flatbuffers/go"
)

type Feature struct {
	builder *flatbuffers.Builder

	geometry   *Geometry
	properties []byte
	columns    []Column
}

func NewFeature(builder *flatbuffers.Builder) *Feature {
	return &Feature{
		builder: builder,
	}
}

func (f *Feature) SetGeometry(geometry *Geometry) *Feature {
	f.geometry = geometry
	return f
}

func (f *Feature) SetProperties(properties []byte) *Feature {
	f.properties = properties
	return f
}

func (f *Feature) SetColumns(columns []Column) *Feature {
	f.columns = columns
	return f
}

func (f *Feature) Build() flatbuffers.UOffsetT {
	// Make sure all columns will be together in the flatbuffer.
	columnOffsetsSlice := make([]flatbuffers.UOffsetT, 0, len(f.columns))
	for i := len(f.columns) - 1; i >= 0; i-- {
		columnOffsetsSlice = append(columnOffsetsSlice, f.columns[i].Build())
	}

	geometryOffset := f.geometry.Build()
	propertiesOffset := f.builder.CreateByteVector(f.properties)

	// columsnOffsetSlice is already in reverse order, so we do not need to
	// reverse it again.
	flattypes.FeatureStartColumnsVector(f.builder, len(columnOffsetsSlice))
	for i := 0; i < len(columnOffsetsSlice); i++ {
		f.builder.PrependUOffsetT(columnOffsetsSlice[i])
	}
	columnsOffset := f.builder.EndVector(len(columnOffsetsSlice))

	flattypes.FeatureStart(f.builder)
	flattypes.FeatureAddGeometry(f.builder, geometryOffset)
	flattypes.FeatureAddProperties(f.builder, propertiesOffset)
	flattypes.FeatureAddColumns(f.builder, columnsOffset)

	return flattypes.FeatureEnd(f.builder)
}

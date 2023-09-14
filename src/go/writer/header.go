package writer

import (
	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	flatbuffers "github.com/google/flatbuffers/go"
)

// Header is the writer responsible for writting the header of the flatgeobuf to
// a given writer. It handles all the flatbuffer details.
type Header struct {
	builder *flatbuffers.Builder

	name          string
	envelope      []float64
	geometryType  flattypes.GeometryType
	hasZ          bool
	hasM          bool
	hasT          bool
	hasTm         bool
	columns       []*Column
	featuresCount uint64
	indexNodeSize uint16
	crs           *Crs
	title         string
	description   string
	metadata      string
}

func NewHeader(builder *flatbuffers.Builder) *Header {
	return &Header{
		builder: builder,
	}
}

func (h *Header) SetName(name string) *Header {
	h.name = name
	return h
}

func (h *Header) SetEnvelope(envelope []float64) *Header {
	h.envelope = envelope
	return h
}

func (h *Header) SetGeometryType(geometryType flattypes.GeometryType) *Header {
	h.geometryType = geometryType
	return h
}

func (h *Header) SetHasZ(hasZ bool) *Header {
	h.hasZ = hasZ
	return h
}

func (h *Header) SetHasM(hasM bool) *Header {
	h.hasM = hasM
	return h
}

func (h *Header) SetHasT(hasT bool) *Header {
	h.hasT = hasT
	return h
}

func (h *Header) SetHasTm(hasTm bool) *Header {
	h.hasTm = hasTm
	return h
}

func (h *Header) SetColumns(columns []*Column) *Header {
	h.columns = columns
	return h
}

func (h *Header) SetFeaturesCount(featuresCount uint64) *Header {
	h.featuresCount = featuresCount
	return h
}

func (h *Header) SetIndexNodeSize(indexNodeSize uint16) *Header {
	h.indexNodeSize = indexNodeSize
	return h
}

func (h *Header) SetCrs(crs *Crs) *Header {
	h.crs = crs
	return h
}

func (h *Header) SetTitle(title string) *Header {
	h.title = title
	return h
}

func (h *Header) SetDescription(description string) *Header {
	h.description = description
	return h
}

func (h *Header) SetMetadata(metadata string) *Header {
	h.metadata = metadata
	return h
}

func (h *Header) Build() flatbuffers.UOffsetT {
	if h.builder == nil {
		return 0
	}

	nameOffset := maybeCreateString(h.builder, h.name)

	crsOffset := h.crs.Build()

	// Make sure all columns will be together in the flatbuffer.
	columnOffsetsSlice := make([]flatbuffers.UOffsetT, 0, len(h.columns))
	for i := len(h.columns) - 1; i >= 0; i-- {
		columnOffsetsSlice = append(columnOffsetsSlice, h.columns[i].Build())
	}

	flattypes.HeaderStartEnvelopeVector(h.builder, len(h.envelope))
	for i := len(h.envelope) - 1; i >= 0; i-- {
		h.builder.PrependFloat64(h.envelope[i])
	}
	envelopeOffset := h.builder.EndVector(len(h.envelope))

	// columsnOffsetSlice is already in reverse order, so we do not need to
	// reverse it again.
	flattypes.HeaderStartColumnsVector(h.builder, len(columnOffsetsSlice))
	for i := 0; i < len(columnOffsetsSlice); i++ {
		h.builder.PrependUOffsetT(columnOffsetsSlice[i])
	}
	columnsOffset := h.builder.EndVector(len(columnOffsetsSlice))

	titleOffset := maybeCreateString(h.builder, h.title)
	descriptionOffset := maybeCreateString(h.builder, h.description)
	metaDataOffset := maybeCreateString(h.builder, h.metadata)

	flattypes.HeaderStart(h.builder)

	flattypes.HeaderAddName(h.builder, nameOffset)
	flattypes.HeaderAddEnvelope(h.builder, envelopeOffset)
	flattypes.HeaderAddGeometryType(h.builder, h.geometryType)
	flattypes.HeaderAddHasZ(h.builder, h.hasZ)
	flattypes.HeaderAddHasM(h.builder, h.hasM)
	flattypes.HeaderAddHasT(h.builder, h.hasT)
	flattypes.HeaderAddHasTm(h.builder, h.hasTm)
	flattypes.HeaderAddColumns(h.builder, columnsOffset)
	flattypes.HeaderAddFeaturesCount(h.builder, h.featuresCount)
	flattypes.HeaderAddIndexNodeSize(h.builder, h.indexNodeSize)
	flattypes.HeaderAddCrs(h.builder, crsOffset)
	flattypes.HeaderAddTitle(h.builder, titleOffset)
	flattypes.HeaderAddDescription(h.builder, descriptionOffset)
	flattypes.HeaderAddMetadata(h.builder, metaDataOffset)

	return flattypes.HeaderEnd(h.builder)
}

func (h *Header) Builder() *flatbuffers.Builder {
	return h.builder
}

package writer

import (
	"github.com/flatgeobuf/flatgeobuf/src/go/flattypes"
	flatbuffers "github.com/google/flatbuffers/go"
)

type Geometry struct {
	builder *flatbuffers.Builder

	ends  []uint32
	xy    []float64
	z     []float64
	m     []float64
	t     []float64
	tm    []uint64
	typ   flattypes.GeometryType
	parts []Geometry
}

func NewGeometry(builder *flatbuffers.Builder) *Geometry {
	return &Geometry{
		builder: builder,
	}
}

func (g *Geometry) SetEnds(ends []uint32) *Geometry {
	g.ends = ends
	return g
}

func (g *Geometry) SetXY(xy []float64) *Geometry {
	g.xy = xy
	return g
}

func (g *Geometry) SetZ(z []float64) *Geometry {
	g.z = z
	return g
}

func (g *Geometry) SetM(m []float64) *Geometry {
	g.m = m
	return g
}

func (g *Geometry) SetT(t []float64) *Geometry {
	g.t = t
	return g
}

func (g *Geometry) SetTM(tm []uint64) *Geometry {
	g.tm = tm
	return g
}

func (g *Geometry) SetType(typ flattypes.GeometryType) *Geometry {
	g.typ = typ
	return g
}

func (g *Geometry) SetParts(parts []Geometry) *Geometry {
	g.parts = parts
	return g
}

func (g *Geometry) Build() flatbuffers.UOffsetT {
	if g.builder == nil {
		return 0
	}

	flattypes.GeometryStartEndsVector(g.builder, len(g.ends))
	for i := len(g.ends) - 1; i >= 0; i-- {
		g.builder.PrependUint32(g.ends[i])
	}
	endsOffset := g.builder.EndVector(len(g.ends))

	flattypes.GeometryStartXyVector(g.builder, len(g.xy))
	for i := len(g.xy) - 1; i >= 0; i-- {
		g.builder.PrependFloat64(g.xy[i])
	}
	xyOffset := g.builder.EndVector(len(g.xy))

	flattypes.GeometryStartZVector(g.builder, len(g.z))
	for i := len(g.z) - 1; i >= 0; i-- {
		g.builder.PrependFloat64(g.z[i])
	}
	zOffset := g.builder.EndVector(len(g.z))

	flattypes.GeometryStartMVector(g.builder, len(g.m))
	for i := len(g.m) - 1; i >= 0; i-- {
		g.builder.PrependFloat64(g.m[i])
	}
	mOffset := g.builder.EndVector(len(g.m))

	flattypes.GeometryStartTVector(g.builder, len(g.t))
	for i := len(g.t) - 1; i >= 0; i-- {
		g.builder.PrependFloat64(g.t[i])
	}
	tOffset := g.builder.EndVector(len(g.t))

	flattypes.GeometryStartTmVector(g.builder, len(g.tm))
	for i := len(g.tm) - 1; i >= 0; i-- {
		g.builder.PrependUint64(g.tm[i])
	}
	tmOffset := g.builder.EndVector(len(g.tm))

	flattypes.GeometryStartPartsVector(g.builder, len(g.parts))
	for i := len(g.parts) - 1; i >= 0; i-- {
		g.builder.PrependUOffsetT(g.parts[i].Build())
	}
	partsOffset := g.builder.EndVector(len(g.parts))

	flattypes.GeometryStart(g.builder)

	flattypes.GeometryAddEnds(g.builder, endsOffset)
	flattypes.GeometryAddXy(g.builder, xyOffset)
	flattypes.GeometryAddZ(g.builder, zOffset)
	flattypes.GeometryAddM(g.builder, mOffset)
	flattypes.GeometryAddT(g.builder, tOffset)
	flattypes.GeometryAddTm(g.builder, tmOffset)
	flattypes.GeometryAddType(g.builder, g.typ)
	flattypes.GeometryAddParts(g.builder, partsOffset)

	return flattypes.GeometryEnd(g.builder)
}

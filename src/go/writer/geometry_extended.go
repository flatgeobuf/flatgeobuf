package writer

import "math"

type GeometryExtended struct {
	*Geometry
}

func NewGeometryExtended(geometry *Geometry) *GeometryExtended {
	return &GeometryExtended{
		geometry,
	}
}

func (g *GeometryExtended) BoundingBox() (minX, minY, maxX, maxY float64) {
	firstPartEnd := len(g.xy)
	if firstPartEnd == 0 {
		// Nothing to do.
		return
	}

	if len(g.ends) != 0 {
		firstPartEnd = int(g.ends[0])
	}
	firstPart := g.xy[0:firstPartEnd]

	minX = math.Inf(1)
	minY = math.Inf(1)
	maxX = math.Inf(-1)
	maxY = math.Inf(-1)

	for i := 0; i < len(firstPart); i += 2 {
		x := firstPart[i]
		y := firstPart[i+1]
		if x < minX {
			minX = x
		}
		if x > maxX {
			maxX = x
		}
		if y < minY {
			minY = y
		}
		if y > maxY {
			maxY = y
		}
	}

	return
}

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
	if len(g.parts) == 0 {
		return geomBoundingBox(g.Geometry)
	}
	minX = math.Inf(1)
	minY = math.Inf(1)
	maxX = math.Inf(-1)
	maxY = math.Inf(-1)
	for i := 0; i < len(g.parts); i++ {
		partMinX, partMinY, partMaxX, partMaxY := geomBoundingBox(&g.parts[i])
		if partMinX < minX {
			minX = partMinX
		}
		if partMinY < minY {
			minY = partMinY
		}
		if partMaxX > maxX {
			maxX = partMaxX
		}
		if partMaxY > maxY {
			maxY = partMaxY
		}
	}
	return
}

func geomBoundingBox(g *Geometry) (minX, minY, maxX, maxY float64) {
	firstPartEnd := len(g.xy)
	if firstPartEnd == 0 {
		// Nothing to do.
		return
	}

	if len(g.ends) != 0 {
		// Ends being the number of point we multiply by 2 to get the last
		// x y coordinates of the first part
		firstPartEnd = int(g.ends[0]) * 2
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

import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometryOl(geometry: Geometry, type: GeometryType, ol: any): ISimpleGeometry {
    const {
        Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryLayout
    } = ol.geom
    const xy = Array.from(geometry.xyArray())
    const ends = geometry.endsArray()
    let olEnds = ends ? Array.from(ends.map(e => e << 1)) : new Uint32Array([xy.length])
    switch (type) {
        case GeometryType.Point:
            return new Point(xy)
        case GeometryType.MultiPoint:
            return new MultiPoint(xy, GeometryLayout.XY)
        case GeometryType.LineString:
            return new LineString(xy, GeometryLayout.XY)
        case GeometryType.MultiLineString:
            return new MultiLineString(xy, GeometryLayout.XY, olEnds)
        case GeometryType.Polygon:
            return new Polygon(xy, GeometryLayout.XY, olEnds)
        case GeometryType.MultiPolygon:
            let lengths = geometry.lengthsArray()
            let olEndss
            let s = 0
            if (lengths) // multipart multipolygon
                olEndss = Array.from(lengths).map(e => olEnds.slice(s, s += e))
            else if (ends) // single part multipolygon with holes
                olEndss = [Array.from(olEnds)]
            else // single part multipolygon without holes
                olEndss = [[xy.length]]
            return new MultiPolygon(xy, GeometryLayout.XY, olEndss)
        default:
            throw Error('Unknown type')
    }
}

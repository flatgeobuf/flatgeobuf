import { GeometryType } from '../header_generated'
import { Feature } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometryOl(feature: Feature, type: GeometryType, ol: any): ISimpleGeometry {
    const {
        Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryLayout
    } = ol.geom
    const xy = Array.from(feature.xyArray())
    const ends = feature.endsArray()
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
            let endss = feature.endssArray()
            let olEndss
            let s = 0
            if (endss) // multipart multipolygon
                olEndss = Array.from(endss).map(e => olEnds.slice(s, s += e))
            else if (ends) // single part multipolygon with holes
                olEndss = [Array.from(olEnds)]
            else // single part multipolygon without holes
                olEndss = [[xy.length]]
            return new MultiPolygon(xy, GeometryLayout.XY, olEndss)
        default:
            throw Error('Unknown type')
    }
}

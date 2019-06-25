import { GeometryType } from '../header_generated'
import { Feature } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometryOl(feature: Feature, type: GeometryType, ol: any): ISimpleGeometry {
    const {
        Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryLayout
    } = ol.geom
    const coords = Array.from(feature.coordsArray())
    const ends = feature.endsArray()
    let olEnds = ends ? Array.from(ends) : new Uint32Array([coords.length])
    switch (type) {
        case GeometryType.Point:
            return new Point(coords)
        case GeometryType.MultiPoint:
            return new MultiPoint(coords, GeometryLayout.XY)
        case GeometryType.LineString:
            return new LineString(coords, GeometryLayout.XY)
        case GeometryType.MultiLineString:
            return new MultiLineString(coords, GeometryLayout.XY, olEnds)
        case GeometryType.Polygon:
            return new Polygon(coords, GeometryLayout.XY, olEnds)
        case GeometryType.MultiPolygon:
            let endss = feature.endssArray()
            let olEndss
            let s = 0
            if (endss) // multipart multipolygon
                olEndss = Array.from(endss).map(e => olEnds.slice(s, s += e))
            else if (ends) // single part multipolygon with holes
                olEndss = [Array.from(olEnds)]
            else // single part multipolygon without holes
                olEndss = [[coords.length]]
            return new MultiPolygon(coords, GeometryLayout.XY, olEndss)
        default:
            throw Error('Unknown type')
    }
}

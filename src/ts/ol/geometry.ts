import { GeometryType } from '../header_generated'
import { Feature } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometryOl(feature: Feature, type: GeometryType, ol: any): ISimpleGeometry {
    const {
        Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryLayout
    } = ol.geom
    const coords = Array.from(feature.coordsArray())
    const ends = feature.endsArray()
    switch (type) {
        case GeometryType.Point:
            return new Point(coords)
        case GeometryType.MultiPoint:
            return new MultiPoint(coords, GeometryLayout.XY)
        case GeometryType.LineString:
            return new LineString(coords, GeometryLayout.XY)
        case GeometryType.MultiLineString:
            return new MultiLineString(coords, GeometryLayout.XY, ends)
        case GeometryType.Polygon:
            return new Polygon(coords, GeometryLayout.XY, ends)
        case GeometryType.MultiPolygon:
            let endss = feature.endssArray()
            let olEnds
            let s = 0
            if (endss)
                olEnds = Array.from(endss).map(e => ends.slice(s, s += e))
            else if (ends)
                olEnds = [Array.from(ends)]
            else
                olEnds = [[coords.length * 2]]
            return new MultiPolygon(coords, GeometryLayout.XY, olEnds)
        default:
            throw Error('Unknown type')
    }
}

import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometryOl(geometry: Geometry, type: GeometryType, ol: any): ISimpleGeometry {
    const {
        Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon, GeometryLayout
    } = ol.geom
    const xy = geometry.xyArray() ? Array.from(geometry.xyArray()) : undefined
    const ends = geometry.endsArray()
    let olEnds: number[] | Uint32Array = undefined
    if (xy)
        olEnds = ends ? Array.from(ends.map(e => e << 1)) : new Uint32Array([xy.length])
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
            const mp = new MultiPolygon([])
            for (let i = 0; i < geometry.partsLength(); i++)
                mp.appendPolygon(createGeometryOl(geometry.parts(i), GeometryType.Polygon, ol))
            return mp
        default:
            throw Error('Unknown type')
    }
}

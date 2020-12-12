import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

import { ISimpleGeometry } from '../generic/geometry'

import Point from 'ol/geom/Point'
import MultiPoint from 'ol/geom/MultiPoint'
import LineString from 'ol/geom/LineString'
import MultiLineString from 'ol/geom/MultiLineString'
import Polygon from 'ol/geom/Polygon'
import MultiPolygon from 'ol/geom/MultiPolygon'
import GeometryLayout from 'ol/geom/GeometryLayout'

export function createGeometryOl(geometry: Geometry | null, type: GeometryType): ISimpleGeometry | undefined {
    if (!geometry)
        return
    const xyArray = geometry.xyArray()
    if (xyArray) {
        const xy = Array.from(geometry.xyArray() as ArrayLike<number>)
        const ends = geometry.endsArray()
        const olEnds = ends ? Array.from(ends.map(e => e << 1)) : [xy.length]
        switch (type) {
            case GeometryType.Point:
                return new Point(xy)
            case GeometryType.MultiPoint:
                return new MultiPoint(xy, 'XY' as GeometryLayout)
            case GeometryType.LineString:
                return new LineString(xy, 'XY' as GeometryLayout)
            case GeometryType.MultiLineString:
                return new MultiLineString(xy, 'XY' as GeometryLayout, olEnds)
            case GeometryType.Polygon:
                return new Polygon(xy, 'XY' as GeometryLayout, olEnds)
        }
    } else if (type === GeometryType.MultiPolygon) {
        const mp = new MultiPolygon([])
        for (let i = 0; i < geometry.partsLength(); i++)
            mp.appendPolygon(createGeometryOl(geometry.parts(i) as Geometry, GeometryType.Polygon) as Polygon)
        return mp
    }
    throw new Error('Unknown type')
}

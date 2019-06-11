import { GeometryType } from '../header_generated'
import { Feature } from '../feature_generated'

import Point from 'ol/geom/Point'
import MultiPoint from 'ol/geom/MultiPoint'
import LineString from 'ol/geom/LineString'
import MultiLineString from 'ol/geom/MultiLineString'
import Polygon from 'ol/geom/Polygon'
import MultiPolygon from 'ol/geom/MultiPolygon'
import GeometryLayout from 'ol/geom/GeometryLayout'

import { ISimpleGeometry } from '../generic/geometry'

export function createGeometry(feature: Feature, type: GeometryType): ISimpleGeometry {
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
            if (!endss)
                return new MultiPolygon(coords, GeometryLayout.XY, [ends])
            let s = 0
            return new MultiPolygon(
                coords,
                GeometryLayout.XY,
                Array.from(endss).map(e => ends.slice(s, s += e)))
        default:
            throw Error('Unknown type')
    }
}

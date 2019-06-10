import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Feature  } from '../feature_generated'

import SimpleGeometry from 'ol/geom/SimpleGeometry'
import Point from 'ol/geom/Point'
import MultiPoint from 'ol/geom/MultiPoint'
import LineString from 'ol/geom/LineString'
import MultiLineString from 'ol/geom/MultiLineString'
import Polygon from 'ol/geom/Polygon'
import MultiPolygon from 'ol/geom/MultiPolygon'

import GeometryLayout from 'ol/geom/GeometryLayout'

export function buildGeometry(builder: flatbuffers.Builder, geometry: SimpleGeometry, type: GeometryType) {
    const { coords, ends, endss } = parseGeometry(geometry, type)
    const coordsOffset = Feature.createCoordsVector(builder, coords)

    let endsOffset: number = null
    let endssOffset: number = null
    if (ends)
        endsOffset = Feature.createEndsVector(builder, ends)
    if (endss)
        endssOffset = Feature.createEndssVector(builder, endss)

    return function() {
        if (endsOffset)
            Feature.addEnds(builder, endsOffset)
        if (endssOffset)
            Feature.addEndss(builder, endssOffset)
        Feature.addCoords(builder, coordsOffset)
    }
}

interface IParsedGeometry {
    coords: number[],
    ends: number[],
    endss: number[]
}

function flat(a: any[]): number[] {
    return a.reduce((acc, val) =>
        Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
}

function parseGeometry(geometry: SimpleGeometry, type: GeometryType) {
    let coords: number[] = geometry.getFlatCoordinates()
    let ends: number[] = null
    let endss: number[] = null
    if (type === GeometryType.MultiLineString || type === GeometryType.Polygon)
        ends = geometry.getEnds()
    if (type === GeometryType.MultiPolygon) {
        ends = flat(geometry.getEndss())
        endss = geometry.getEndss().map(ends => ends.length)
    }
    return {
        coords,
        ends,
        endss
    } as IParsedGeometry
}

function pairFlatCoordinates(coordinates: Float64Array) {
    const newArray: number[][] = []
    for (let i = 0; i < coordinates.length; i += 2)
        newArray.push([coordinates[i], coordinates[i + 1]])
    return newArray
}

export function toSimpleGeometry(feature: Feature, type: GeometryType) {
    const coords = Array.from(feature.coordsArray())
    const ends = feature.endsArray()
    let geometry
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
    return geometry
}

export function toGeometryType(name: string) {
    const type: GeometryType = (GeometryType as any)[name]
    return type
}

import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Feature  } from '../feature_generated'

import { IParsedGeometry, flat, pairFlatCoordinates } from '../generic/geometry'

export interface IGeoJsonGeometry {
    type: string
    coordinates: number[] | number[][] | number[][][] | number[][][][]
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: IGeoJsonGeometry) {
    const { xy, ends, lengths } = parseGeometry(geometry)
    const coordsOffset = Feature.createXyVector(builder, xy)

    let endsOffset: number = null
    let lengthsOffset: number = null
    if (ends)
        endsOffset = Feature.createEndsVector(builder, ends)
    if (lengths)
        lengthsOffset = Feature.createLengthsVector(builder, lengths)

    return function() {
        if (endsOffset)
            Feature.addEnds(builder, endsOffset)
        if (lengthsOffset)
            Feature.addLengths(builder, lengthsOffset)
        Feature.addXy(builder, coordsOffset)
    }
}

function parseGeometry(geometry: IGeoJsonGeometry) {
    const cs = geometry.coordinates
    let xy: number[] = null
    let ends: number[] = null
    let lengths: number[] = null
    let end = 0
    switch (geometry.type) {
        case 'Point':
            xy = cs as number[]
            break
        case 'MultiPoint':
        case 'LineString':
            xy = flat(cs as number[][])
            break
        case 'MultiLineString':
        case 'Polygon':
            const css = cs as number[][][]
            xy = flat(css)
            if (css.length > 1)
                ends = css.map(c => end += c.length)
            break
        case 'MultiPolygon':
            const csss = cs as number[][][][]
            xy = flat(csss)
            if (csss.length > 1) {
                lengths = csss.map(c => c.length)
                ends = flat(csss.map(cc => cc.map(c => end += c.length)))
            } else
                if (csss[0].length > 1)
                    ends = csss[0].map(c => end += c.length)
            break
    }
    return {
        xy,
        ends,
        lengths
    } as IParsedGeometry
}

function extractParts(xy: Float64Array, ends: Uint32Array) {
    if (!ends)
        return [pairFlatCoordinates(xy)]
    let s = 0
    let xySlices = Array.from(ends)
        .map(e => xy.slice(s, s = e << 1))
    return xySlices
        .map(cs => pairFlatCoordinates(cs))
}

function extractPartsParts(
    xy: Float64Array,
        ends: Uint32Array,
        lengths: Uint32Array) {
    if (!lengths)
        return [extractParts(xy, ends)]
    let s = 0
    let xySlices = Array.from(ends)
        .map(e => xy.slice(s, s = e << 1))
    s = 0
    return Array.from(lengths)
        .map(e => xySlices.slice(s, s += e)
        .map(cs => pairFlatCoordinates(cs)))
}

function toGeoJsonCoordinates(feature: Feature, type: GeometryType) {
    const xy = feature.xyArray()
    switch (type) {
        case GeometryType.Point:
            return Array.from(xy)
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(xy)
        case GeometryType.MultiLineString:
            return extractParts(xy, feature.endsArray())
        case GeometryType.Polygon:
            return extractParts(xy, feature.endsArray())
        case GeometryType.MultiPolygon:
            return extractPartsParts(xy,
                feature.endsArray(),
                feature.lengthsArray())
    }
}

export function fromGeometry(feature: Feature, type: GeometryType) {
    const coordinates = toGeoJsonCoordinates(feature, type)
    return {
        type: GeometryType[type],
        coordinates,
    } as IGeoJsonGeometry
}
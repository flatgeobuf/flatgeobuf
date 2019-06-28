import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Feature  } from '../feature_generated'

import { IParsedGeometry, flat, pairFlatCoordinates } from '../generic/geometry'

export interface IGeoJsonGeometry {
    type: string
    coordinates: number[] | number[][] | number[][][] | number[][][][]
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: IGeoJsonGeometry) {
    const { coords, ends, endss } = parseGeometry(geometry)
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

function parseGeometry(geometry: IGeoJsonGeometry) {
    const cs = geometry.coordinates
    let coords: number[] = null
    let ends: number[] = null
    let endss: number[] = null
    let end = 0
    let endend = 0
    switch (geometry.type) {
        case 'Point': {
            coords = cs as number[]
            break
        }
        case 'MultiPoint':
        case 'LineString': {
            coords = flat(cs as number[][])
            break
        }
        case 'MultiLineString': {
            const css = cs as number[][][]
            coords = flat(css)
            if (css.length > 1)
                ends = css.map(c => end += c.length * 2)
            break
        }
        case 'Polygon': {
            const css = cs as number[][][]
            coords = flat(css)
            if (css.length > 1)
                ends = css.map(c => end += c.length * 2)
            break
        }
        case 'MultiPolygon': {
            const csss = cs as number[][][][]
            coords = flat(csss)
            if (csss.length > 1) {
                endss = csss.map(c => endend += c.length)
                ends = flat(csss.map(cc => cc.map(c => end += c.length * 2)))
            } else
                if (csss[0].length > 1)
                    ends = csss[0].map(c => end += c.length * 2)
            break
        }
    }
    return {
        coords,
        ends,
        endss
    } as IParsedGeometry
}

function extractParts(coords: Float64Array, ends: Uint32Array) {
    if (!ends)
        return [pairFlatCoordinates(coords)]
    let s = 0
    let coordsSlices = Array.from(ends)
        .map(e => coords.slice(s, s = e))
    return coordsSlices
        .map(cs => pairFlatCoordinates(cs))
}

function extractPartsParts(
        coords: Float64Array,
        ends: Uint32Array,
        endss: Uint32Array) {
    if (!endss)
        return [extractParts(coords, ends)]
    let s = 0
    let coordsSlices = Array.from(ends)
        .map(e => coords.slice(s, s = e))
    s = 0
    return Array.from(endss)
        .map(e => coordsSlices.slice(s, s = e)
        .map(cs => pairFlatCoordinates(cs)))
}

function toGeoJsonCoordinates(feature: Feature, type: GeometryType) {
    const coords = feature.coordsArray()
    switch (type) {
        case GeometryType.Point:
            return Array.from(coords)
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(coords)
        case GeometryType.MultiLineString:
            return extractParts(coords, feature.endsArray())
        case GeometryType.Polygon:
            return extractParts(coords, feature.endsArray())
        case GeometryType.MultiPolygon:
            return extractPartsParts(coords,
                feature.endsArray(),
                feature.endssArray())
    }
}

export function fromGeometry(feature: Feature, type: GeometryType) {
    const coordinates = toGeoJsonCoordinates(feature, type)
    return {
        type: GeometryType[type],
        coordinates,
    } as IGeoJsonGeometry
}
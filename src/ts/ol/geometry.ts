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
    const { coords, lengths, ringLengths, ringCounts } = parseGeometry(geometry, type)
    const coordsOffset = Feature.createCoordsVector(builder, coords)

    let lengthsOffset: number = null
    let ringLengthsOffset: number = null
    let ringCountsOffset: number = null
    if (lengths)
        lengthsOffset = Feature.createLengthsVector(builder, lengths)
    if (ringLengths)
        ringLengthsOffset = Feature.createRingLengthsVector(builder, ringLengths)
    if (ringCounts)
        ringCountsOffset = Feature.createRingCountsVector(builder, ringCounts)

    return function() {
        if (lengthsOffset)
            Feature.addLengths(builder, lengthsOffset)
        if (ringLengths)
            Feature.addRingLengths(builder, ringLengthsOffset)
        if (ringCounts)
            Feature.addRingCounts(builder, ringCountsOffset)
        Feature.addCoords(builder, coordsOffset)
    }
}

interface IParsedGeometry {
    coords: number[],
    lengths: number[],
    ringLengths: number[],
    ringCounts: number[]
}

function flat(a: any[]): number[] {
    return a.reduce((acc, val) =>
        Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
}

function parseGeometry(geometry: SimpleGeometry, type: GeometryType) {
    let coords: number[] = geometry.getFlatCoordinates()
    let lengths: number[] = null
    let ringLengths: number[] = null
    let ringCounts: number[] = null
    switch (type) {
        case GeometryType.MultiLineString:
            lengths = geometry.getEnds()
            break
        case GeometryType.Polygon:
            ringLengths = geometry.getEnds()
            break
        case GeometryType.MultiPolygon: {
            const endss = geometry.getEndss()
            lengths = endss.map(ends => ends.reduce((acc, c) => acc + c))
            ringLengths = flat(endss)
            ringCounts = endss.map(ends => ends.length)
            break
        }
    }
    return {
        coords,
        lengths,
        ringLengths,
        ringCounts,
    } as IParsedGeometry
}

function pairFlatCoordinates(coordinates: Float64Array) {
    const newArray: number[][] = []
    for (let i = 0; i < coordinates.length; i += 2)
        newArray.push([coordinates[i], coordinates[i + 1]])
    return newArray
}

function extractParts(coords: Float64Array, lengths: Uint32Array) {
    if (!lengths)
        return [pairFlatCoordinates(coords)]
    const parts = []
    let offset = 0
    for (const length of lengths) {
        const slice = coords.slice(offset, offset + length)
        parts.push(pairFlatCoordinates(slice))
        offset += length
    }
    return parts
}

function extractPartsParts(
        coords: Float64Array,
        lengths: Uint32Array,
        ringLengths: Uint32Array,
        ringCounts: Uint32Array) {
    if (!lengths)
        return [extractParts(coords, ringLengths)]
    const parts = []
    let offset = 0
    let ringLengthsOffset = 0
    for (let i = 0; i < lengths.length; i++) {
        const length = lengths[i]
        const ringCount = ringCounts[i]
        const slice = coords.slice(offset, offset + length)
        const ringLengthsSlice = ringLengths.slice(ringLengthsOffset, ringLengthsOffset + ringCount)
        parts.push(extractParts(slice, ringLengthsSlice))
        offset += length
        ringLengthsOffset += ringCount
    }
    return parts
}

export function toSimpleGeometry(feature: Feature, type: GeometryType) {
    const coords = feature.coordsArray()
    const lengths = feature.lengthsArray()
    const ringLenghts = feature.ringLengthsArray()

    let geometry
    switch (type) {
        case GeometryType.Point:
            geometry = new Point(coords)
            break
        case GeometryType.MultiPoint:
            geometry = new MultiPoint(Array.from(coords), GeometryLayout.XY)
            break
        case GeometryType.LineString:
            geometry = new LineString(Array.from(coords), GeometryLayout.XY)
            break
        case GeometryType.MultiLineString:
            geometry = new MultiLineString(Array.from(coords), GeometryLayout.XY, Array.from(lengths))
            break
        case GeometryType.Polygon:
            geometry = new Polygon(Array.from(coords), GeometryLayout.XY, ringLenghts)
            break
        case GeometryType.MultiPolygon:
            geometry = new MultiPolygon(Array.from(coords), GeometryLayout.XY, ringLenghts)
            break
    }
    return geometry
}

export function toGeometryType(name: string) {
    const type: GeometryType = (GeometryType as any)[name]
    return type
}

import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

export interface IGeoJsonGeometry {
    type: string
    coordinates: number[]
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: IGeoJsonGeometry) {
    const { coords, lengths, ringLengths, ringCounts } = parseGeometry(geometry)
    const coordsOffset = Geometry.createCoordsVector(builder, coords)

    let lengthsOffset = null
    let ringLengthsOffset = null
    let ringCountsOffset = null
    if (lengths)
        lengthsOffset = Geometry.createLengthsVector(builder, lengths)
    if (ringLengths)
        ringLengthsOffset = Geometry.createRingLengthsVector(builder, ringLengths)
    if (ringCounts)
        ringCountsOffset = Geometry.createRingCountsVector(builder, ringCounts)

    Geometry.startGeometry(builder)
    if (lengthsOffset)
        Geometry.addLengths(builder, lengthsOffset)
    if (ringLengths)
        Geometry.addRingLengths(builder, ringLengthsOffset)
    if (ringCounts)
        Geometry.addRingCounts(builder, ringCountsOffset)
    Geometry.addCoords(builder, coordsOffset)
    const offset = Geometry.endGeometry(builder)

    return offset
}

interface IParsedGeometry {
    coords: number[],
    lengths: number[],
    ringLengths: number[],
    ringCounts: number[]
}

function flat(a) {
    return a.reduce((acc, val) =>
        Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
 }

function parseGeometry(geometry: any): IParsedGeometry {
    const cs = geometry.coordinates
    let coords = null
    let lengths = null
    let ringLengths = null
    let ringCounts = null
    switch (geometry.type) {
        case 'Point':
            coords = cs
            break
        case 'MultiPoint':
        case 'LineString':
            coords = flat(cs)
            break
        case 'MultiLineString':
            coords = flat(cs)
            if (cs.length > 1)
                lengths = cs.map(c => c.length * 2)
            break
        case 'Polygon':
            coords = flat(cs)
            if (cs.length > 1)
                ringLengths = cs.map(c => c.length * 2)
            break
        case 'MultiPolygon':
            coords = flat(cs)
            if (cs.length > 1) {
                lengths = cs.map(cc => cc.map(c => c.length * 2).reduce((a, b) => a + b, 0))
                ringCounts = cs.map(c => c.length)
                ringLengths = flat(cs.map(cc => cc.map(c => c.length * 2)))
            } else
                if (cs[0].length > 1)
                    ringLengths = cs[0].map(c => c.length * 2)
            break
    }
    return {
        coords,
        lengths,
        ringLengths,
        ringCounts,
    }
}

function pairFlatCoordinates(coordinates: Float64Array) {
    const newArray = []
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
        parts.push(extractParts(slice, (ringLengthsSlice.length > 0 ? ringLengthsSlice : null)))
        offset += length
        ringLengthsOffset += ringCount
    }
    return parts
}

function toGeoJsonCoordinates(geometry: FlatGeobuf.Geometry, type: FlatGeobuf.GeometryType) {
    const coords = geometry.coordsArray()
    switch (type) {
        case GeometryType.Point:
            return Array.from(coords)
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(coords)
        case GeometryType.MultiLineString:
            return extractParts(coords, geometry.lengthsArray())
        case GeometryType.Polygon:
            return extractParts(coords, geometry.ringLengthsArray())
        case GeometryType.MultiPolygon:
            return extractPartsParts(coords,
                geometry.lengthsArray(),
                geometry.ringLengthsArray(),
                geometry.ringCountsArray())
    }
}

export function fromGeometry(
        geometry: FlatGeobuf.Geometry,
        type: FlatGeobuf.GeometryType): IGeoJsonGeometry {
    const coordinates = toGeoJsonCoordinates(geometry, type)
    return {
        type: GeometryType[type],
        coordinates,
    }
}

export function toGeometryType(name: string): FlatGeobuf.GeometryType {
    const type: FlatGeobuf.GeometryType = (GeometryType as any)[name]
    return type
}

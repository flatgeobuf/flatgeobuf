import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

export function buildGeometry(builder: flatbuffers.Builder, geometry: any) {
    const { coords, lengths } = parseGeometry(geometry)
    const coordsOffset = Geometry.createCoordsVector(builder, coords)
    let lengthsOffset = null
    if (lengths)
        lengthsOffset = Geometry.createLengthsVector(builder, lengths)
    Geometry.startGeometry(builder)
    if (lengthsOffset)
        Geometry.addLengths(builder, lengthsOffset)
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
    return a.reduce((acc, val) => Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
 }

function parseGeometry(geometry: any): IParsedGeometry {
    const cs = geometry.coordinates

    let coords = null
    let lengths = null
    const ringLengths = null
    const ringCounts = null
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

function toGeoJsonCoordinates(geometry: FlatGeobuf.Geometry, type: FlatGeobuf.GeometryType) {
    const coords = geometry.coordsArray()
    const lengths = geometry.lengthsLength()
    switch (type) {
        case GeometryType.Point:
            return Array.from(coords)
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(coords)
        case GeometryType.MultiLineString:
            if (lengths) {
                const parts = []
                let offset = 0
                for (let i = 0; i < lengths; i++) {
                    const length = geometry.lengths(i)
                    const slice = coords.slice(offset, offset + length)
                    parts.push(pairFlatCoordinates(slice))
                    offset += length
                }
                return parts
            } else
                return [pairFlatCoordinates(coords)]
    }
}

export function fromGeometry(geometry: FlatGeobuf.Geometry, type: FlatGeobuf.GeometryType) {
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

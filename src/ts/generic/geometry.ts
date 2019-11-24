import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

export interface IParsedGeometry {
    xy: number[],
    ends: number[],
    lengths: number[]
}

export interface ISimpleGeometry {
    getFlatCoordinates(): number[]
}

export interface IPolygon extends ISimpleGeometry {
    getEnds(): number[]
}

export interface IMultiLineString extends ISimpleGeometry {
    getEnds(): number[]
}

export interface IMultiPolygon extends ISimpleGeometry {
    getEndss(): number[][]
}

export interface ICreateGeometry {
    (geometry: Geometry, type: GeometryType): ISimpleGeometry;
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: ISimpleGeometry, type: GeometryType) {
    const { xy, ends, lengths } = parseGeometry(geometry, type)
    const xyOffset = Geometry.createXyVector(builder, xy)

    let endsOffset: number = null
    let lengthsOffset: number = null
    if (ends)
        endsOffset = Geometry.createEndsVector(builder, ends)
    if (lengths)
        lengthsOffset = Geometry.createLengthsVector(builder, lengths)

    return function () {
        if (endsOffset)
            Geometry.addEnds(builder, endsOffset)
        if (lengthsOffset)
            Geometry.addLengths(builder, lengthsOffset)
        Geometry.addXy(builder, xyOffset)
    }
}

export function flat(a: any[]): number[] {
    return a.reduce((acc, val) =>
        Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
}

export function parseGeometry(geometry: ISimpleGeometry, type: GeometryType) {
    let xy: number[] = geometry.getFlatCoordinates()
    let ends: number[] = null
    let lengths: number[] = null
    if (type === GeometryType.MultiLineString) {
        const mlsEnds = (geometry as IMultiLineString).getEnds()
        if (mlsEnds.length > 1)
            ends = mlsEnds.map(e => e >> 1)
    } else if (type === GeometryType.Polygon) {
        const pEnds = (geometry as IPolygon).getEnds()
        if (pEnds.length > 1)
            ends = pEnds.map(e => e >> 1)
    } else if (type === GeometryType.MultiPolygon) {
        const nestedEnds = (geometry as IMultiPolygon).getEndss()
        if (nestedEnds.length > 1 || nestedEnds[0].length > 1)
            ends = flat(nestedEnds).map(e => e >> 1)
        if (nestedEnds.length > 1)
            lengths = nestedEnds.map(ends => ends.length)
    }
    return {
        xy,
        ends,
        lengths
    } as IParsedGeometry
}

export function pairFlatCoordinates(coordinates: Float64Array) {
    const newArray: number[][] = []
    for (let i = 0; i < coordinates.length; i += 2)
        newArray.push([coordinates[i], coordinates[i + 1]])
    return newArray
}

export function toGeometryType(name: string) {
    const type: GeometryType = (GeometryType as any)[name]
    return type
}

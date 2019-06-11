import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Feature } from '../feature_generated'

export interface IParsedGeometry {
    coords: number[],
    ends: number[],
    endss: number[]
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
    (feature: Feature, type: GeometryType): ISimpleGeometry;
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: ISimpleGeometry, type: GeometryType) {
    const { coords, ends, endss } = parseGeometry(geometry, type)
    const coordsOffset = Feature.createCoordsVector(builder, coords)

    let endsOffset: number = null
    let endssOffset: number = null
    if (ends)
        endsOffset = Feature.createEndsVector(builder, ends)
    if (endss)
        endssOffset = Feature.createEndssVector(builder, endss)

    return function () {
        if (endsOffset)
            Feature.addEnds(builder, endsOffset)
        if (endssOffset)
            Feature.addEndss(builder, endssOffset)
        Feature.addCoords(builder, coordsOffset)
    }
}

export function flat(a: any[]): number[] {
    return a.reduce((acc, val) =>
        Array.isArray(val) ? acc.concat(flat(val)) : acc.concat(val), [])
}

export function parseGeometry(geometry: ISimpleGeometry, type: GeometryType) {
    let coords: number[] = geometry.getFlatCoordinates()
    let ends: number[] = null
    let endss: number[] = null
    if (type === GeometryType.MultiLineString)
        ends = (geometry as IMultiLineString).getEnds()
    else if (type === GeometryType.Polygon)
        ends = (geometry as IPolygon).getEnds()
    else if (type === GeometryType.MultiPolygon) {
        ends = flat((geometry as IMultiPolygon).getEndss())
        endss = (geometry as IMultiPolygon).getEndss().map(ends => ends.length)
    }
    return {
        coords,
        ends,
        endss
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

import { flatbuffers } from 'flatbuffers'
import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

export interface IParsedGeometry {
    xy: number[],
    z: number[],
    ends: number[],
    parts: IParsedGeometry[],
    type: GeometryType
}

export interface ISimpleGeometry {
    getFlatCoordinates?(): number[]
    getType(): string
}

export interface IPolygon extends ISimpleGeometry {
    getEnds(): number[]
}

export interface IMultiLineString extends ISimpleGeometry {
    getEnds(): number[]
}

export interface IMultiPolygon extends ISimpleGeometry {
    getEndss(): number[][]
    getPolygons(): IPolygon[]
}

export interface ICreateGeometry {
    (geometry: Geometry, type: GeometryType): ISimpleGeometry;
}

export function buildGeometry(builder: flatbuffers.Builder, parsedGeometry: IParsedGeometry): any {
    const { xy, z, ends, parts, type } = parsedGeometry

    if (parts) {
        const partOffsets = parts.map(part => buildGeometry(builder, part))
        const partsOffset = Geometry.createPartsVector(builder, partOffsets)
        Geometry.start(builder)
        Geometry.addParts(builder, partsOffset)
        return Geometry.end(builder)
    }

    const xyOffset = Geometry.createXyVector(builder, xy)
    let zOffset: number | undefined
    if (z)
        zOffset = Geometry.createZVector(builder, z)

    let endsOffset: number | undefined
    if (ends)
        endsOffset = Geometry.createEndsVector(builder, ends)

    Geometry.start(builder)
    if (endsOffset)
        Geometry.addEnds(builder, endsOffset)
    Geometry.addXy(builder, xyOffset)
    if (zOffset)
        Geometry.addZ(builder, zOffset)
    Geometry.addType(builder, type)
    return Geometry.end(builder)
}

export function flat(a: any[], xy: number[], z: number[]): number[] | undefined {
    if (a.length === 0)
        return
    if (Array.isArray(a[0])) {
        for (const sa of a)
            flat(sa, xy, z)
    } else {
        if (a.length === 2)
            xy.push(...a)
        else {
            xy.push(a[0], a[1])
            z.push(a[2])
        }
    }
}

export function parseGeometry(geometry: ISimpleGeometry, type: GeometryType): IParsedGeometry {
    let xy: number[] | undefined
    let ends: number[] | undefined
    let parts: IParsedGeometry[] | undefined
    if (type === GeometryType.MultiLineString) {
        if (geometry.getFlatCoordinates)
            xy = geometry.getFlatCoordinates()
        const mlsEnds = (geometry as IMultiLineString).getEnds()
        if (mlsEnds.length > 1)
            ends = mlsEnds.map(e => e >> 1)
    } else if (type === GeometryType.Polygon) {
        if (geometry.getFlatCoordinates)
            xy = geometry.getFlatCoordinates()
        const pEnds = (geometry as IPolygon).getEnds()
        if (pEnds.length > 1)
            ends = pEnds.map(e => e >> 1)
    } else if (type === GeometryType.MultiPolygon) {
        const mp = (geometry as IMultiPolygon)
        parts = mp.getPolygons().map(p => parseGeometry(p, GeometryType.Polygon))
    } else {
        if (geometry.getFlatCoordinates)
            xy = geometry.getFlatCoordinates()
    }
    return {
        xy,
        ends,
        type,
        parts
    } as IParsedGeometry
}

export function pairFlatCoordinates(xy: Float64Array, z: Float64Array | undefined): number[][] {
    const newArray: number[][] = []
    for (let i = 0; i < xy.length; i += 2) {
        const a = [xy[i], xy[i + 1]]
        if (z)
            a.push(z[i >> 1])
        newArray.push(a)
    }
    return newArray
}

export function toGeometryType(name: string | undefined): GeometryType {
    if (!name)
        return GeometryType.Unknown
    const type: GeometryType = (GeometryType as any)[name]
    return type
}

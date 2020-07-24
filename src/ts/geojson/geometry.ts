import { GeometryType } from '../header_generated'
import { Geometry } from '../feature_generated'

import { IParsedGeometry, flat, pairFlatCoordinates, toGeometryType } from '../generic/geometry'

export interface IGeoJsonGeometry {
    type: string
    coordinates: number[] | number[][] | number[][][] | number[][][][]
    geometries?: IGeoJsonGeometry[]
}

export function parseGeometry(geometry: IGeoJsonGeometry): IParsedGeometry {
    const cs = geometry.coordinates
    const xy: number[] = []
    const z: number[] = []
    let ends: number[] = null
    let parts: IParsedGeometry[] = null
    const type: GeometryType = toGeometryType(geometry.type)
    let end = 0
    switch (geometry.type) {
        case 'Point':
            flat(cs, xy, z)
            break
        case 'MultiPoint':
        case 'LineString':
            flat(cs as number[][], xy, z)
            break
        case 'MultiLineString':
        case 'Polygon': {
            const css = cs as number[][][]
            flat(css, xy, z)
            if (css.length > 1)
                ends = css.map(c => end += c.length)
            break
        }
        case 'MultiPolygon': {
            const csss = cs as number[][][][]
            const geometries = csss.map(coordinates => ({ type: 'Polygon', coordinates }))
            parts = geometries.map(parseGeometry)
            break
        }
        case 'GeometryCollection':
            parts = geometry.geometries.map(parseGeometry)
            break
    }
    return {
        xy,
        z: z.length > 0 ? z : undefined,
        ends,
        type,
        parts
    } as IParsedGeometry
}

function extractParts(xy: Float64Array, z: Float64Array, ends: Uint32Array) {
    if (!ends || ends.length === 0)
        return [pairFlatCoordinates(xy, z)]
    let s = 0
    const xySlices = Array.from(ends)
        .map(e => xy.slice(s, s = e << 1))
    let zSlices: Float64Array[] = null
    if (z) {
        s = 0
        zSlices = Array.from(ends).map(e => z.slice(s, s = e))
    }
    return xySlices
        .map((xy, i) => pairFlatCoordinates(xy, z ? zSlices[i] : undefined))
}

function toGeoJsonCoordinates(geometry: Geometry, type: GeometryType) {
    const xy = geometry.xyArray()
    const z = geometry.zArray()
    switch (type) {
        case GeometryType.Point: {
            const a = Array.from(xy)
            if (z)
                a.push(z[0])
            return a
        }
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(xy, z)
        case GeometryType.MultiLineString:
            return extractParts(xy, z, geometry.endsArray())
        case GeometryType.Polygon:
            return extractParts(xy, z, geometry.endsArray())
    }
}

export function fromGeometry(geometry: Geometry, type: GeometryType): IGeoJsonGeometry {
    if (type === GeometryType.GeometryCollection) {
        const geometries = []
        for (let i = 0; i < geometry.partsLength(); i++) {
            const part = geometry.parts(i)
            const partType = part.type()
            geometries.push(fromGeometry(part, partType))
        }
        return {
            type: GeometryType[type],
            geometries
        } as IGeoJsonGeometry
    } else if (type === GeometryType.MultiPolygon) {
        const geometries = []
        for (let i = 0; i < geometry.partsLength(); i++)
            geometries.push(fromGeometry(geometry.parts(i), GeometryType.Polygon))
        return {
            type: GeometryType[type],
            coordinates: geometries.map(g => g.coordinates)
        } as IGeoJsonGeometry
    }
    const coordinates = toGeoJsonCoordinates(geometry, type)
    return {
        type: GeometryType[type],
        coordinates
    } as IGeoJsonGeometry
}
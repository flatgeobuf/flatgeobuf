import type {
    Geometry as GeoJsonGeometry,
    GeometryCollection,
    LineString,
    MultiLineString,
    MultiPoint,
    MultiPolygon,
    Point,
    Polygon,
} from 'geojson';
import type { Geometry } from '../flat-geobuf/geometry.js';
import { GeometryType } from '../flat-geobuf/geometry-type.js';

import { flat, type IParsedGeometry, pairFlatCoordinates, toGeometryType } from '../generic/geometry.js';

export interface IGeoJsonGeometry {
    type: string;
    coordinates: number[] | number[][] | number[][][] | number[][][][];
    geometries?: IGeoJsonGeometry[];
}

export function parseGeometry(
    geometry: Point | MultiPoint | LineString | MultiLineString | Polygon | MultiPolygon,
): IParsedGeometry {
    const cs = geometry.coordinates;
    const xy: number[] = [];
    const z: number[] = [];
    const m: number[] = [];
    let ends: number[] | undefined;
    let parts: IParsedGeometry[] | undefined;
    const type: GeometryType = toGeometryType(geometry.type);
    let end = 0;
    switch (geometry.type) {
        case 'Point':
            flat(cs as number[], xy, z, m);
            break;
        case 'MultiPoint':
        case 'LineString':
            flat(cs as number[][], xy, z, m);
            break;
        case 'MultiLineString':
        case 'Polygon': {
            const css = cs as number[][];
            flat(css, xy, z, m);
            if (css.length > 1) ends = css.map((c) => (end += c.length));
            break;
        }
        case 'MultiPolygon': {
            const csss = cs as number[][][][];
            const geometries = csss.map((coordinates) => ({
                type: 'Polygon',
                coordinates,
            })) as Polygon[];
            parts = geometries.map(parseGeometry);
            break;
        }
    }
    return {
        xy,
        z: z.length > 0 ? z : undefined,
        m: m.length > 0 ? m : undefined,
        ends,
        type,
        parts,
    } as IParsedGeometry;
}

export function parseGC(geometry: GeometryCollection): IParsedGeometry {
    const type: GeometryType = toGeometryType(geometry.type);
    const parts: IParsedGeometry[] = [];
    for (let i = 0; i < geometry.geometries.length; i++) {
        const g = geometry.geometries[i];
        if (g.type === 'GeometryCollection') parts.push(parseGC(g));
        else parts.push(parseGeometry(g));
    }
    return {
        type,
        parts,
    } as IParsedGeometry;
}

function extractParts(xy: Float64Array, z: Float64Array | null, m: Float64Array | null, ends: Uint32Array | null) {
    if (!ends || ends.length === 0) return [pairFlatCoordinates(xy, z, m)];
    let s = 0;
    const xySlices = Array.from(ends).map((e) => xy.slice(s, (s = e << 1)));
    let zSlices: Float64Array[] | undefined;
    let mSlices: Float64Array[] | undefined;
    if (z) {
        s = 0;
        zSlices = Array.from(ends).map((e) => z.slice(s, (s = e)));
    }
    if (m) {
        s = 0;
        mSlices = Array.from(ends).map((e) => m.slice(s, (s = e)));
    }
    return xySlices.map((xy, i) => pairFlatCoordinates(xy, zSlices?.[i], mSlices?.[i]));
}

function toGeoJsonCoordinates(geometry: Geometry, type: GeometryType) {
    const xy = geometry.xyArray() as Float64Array;
    const z = geometry.zArray();
    const m = geometry.mArray();
    switch (type) {
        case GeometryType.Point: {
            const a: (number | null)[] = Array.from(xy);
            if (z) a.push(z[0]);
            else if (m) a.push(null);
            if (m) a.push(m[0]);
            return a;
        }
        case GeometryType.MultiPoint:
        case GeometryType.LineString:
            return pairFlatCoordinates(xy, z, m);
        case GeometryType.MultiLineString:
            return extractParts(xy, z, m, geometry.endsArray());
        case GeometryType.Polygon:
            return extractParts(xy, z, m, geometry.endsArray());
    }
}

export function fromGeometry(geometry: Geometry, headerType: GeometryType): GeoJsonGeometry {
    let type = headerType;
    if (type === GeometryType.Unknown) {
        type = geometry.type();
    }
    if (type === GeometryType.GeometryCollection) {
        const geometries: GeoJsonGeometry[] = [];
        for (let i = 0; i < geometry.partsLength(); i++) {
            const part = geometry.parts(i) as Geometry;
            const partType = part.type() as GeometryType;
            geometries.push(fromGeometry(part, partType));
        }
        return {
            type: GeometryType[type],
            geometries,
        } as GeoJsonGeometry;
    }
    if (type === GeometryType.MultiPolygon) {
        const geometries: GeoJsonGeometry[] = [];
        for (let i = 0; i < geometry.partsLength(); i++)
            geometries.push(fromGeometry(geometry.parts(i) as Geometry, GeometryType.Polygon));
        return {
            type: GeometryType[type],
            coordinates: geometries.map((g) => (g as Polygon).coordinates),
        } as GeoJsonGeometry;
    }
    const coordinates = toGeoJsonCoordinates(geometry, type);
    return {
        type: GeometryType[type],
        coordinates,
    } as GeoJsonGeometry;
}

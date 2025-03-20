import type * as flatbuffers from 'flatbuffers';
import type { GeometryLayout } from 'ol/geom/Geometry';
import { GeometryType } from '../flat-geobuf/geometry-type.js';
import { Geometry } from '../flat-geobuf/geometry.js';

export interface IParsedGeometry {
    xy: number[];
    z?: number[];
    m?: number[];
    ends: number[];
    parts: IParsedGeometry[];
    type: GeometryType;
}

export interface ISimpleGeometry {
    getFlatCoordinates?(): number[];
    getType(): string;
    getLayout?: () => GeometryLayout;
}

export interface IPolygon extends ISimpleGeometry {
    getEnds(): number[];
}

export interface IMultiLineString extends ISimpleGeometry {
    getEnds(): number[];
}

export interface IMultiPolygon extends ISimpleGeometry {
    getEndss(): number[][];
    getPolygons(): IPolygon[];
}

export type ICreateGeometry = (geometry: Geometry | null, type: GeometryType) => ISimpleGeometry | undefined;

export function buildGeometry(builder: flatbuffers.Builder, parsedGeometry: IParsedGeometry) {
    const { xy, z, m, ends, parts, type } = parsedGeometry;

    if (parts) {
        const partOffsets = parts.map((part) => buildGeometry(builder, part));
        const partsOffset = Geometry.createPartsVector(builder, partOffsets);
        Geometry.startGeometry(builder);
        Geometry.addParts(builder, partsOffset);
        Geometry.addType(builder, type);
        return Geometry.endGeometry(builder);
    }

    const xyOffset = Geometry.createXyVector(builder, xy);
    let zOffset: number | undefined;
    if (z) zOffset = Geometry.createZVector(builder, z);

    let mOffset: number | undefined;
    if (m) mOffset = Geometry.createMVector(builder, m);

    let endsOffset: number | undefined;
    if (ends) endsOffset = Geometry.createEndsVector(builder, ends);

    Geometry.startGeometry(builder);
    if (endsOffset) Geometry.addEnds(builder, endsOffset);
    Geometry.addXy(builder, xyOffset);
    if (zOffset) Geometry.addZ(builder, zOffset);
    if (mOffset) Geometry.addM(builder, mOffset);
    Geometry.addType(builder, type);
    return Geometry.endGeometry(builder);
}

export function flat(a: number[] | number[][], xy: number[], z: number[]): number[] | undefined {
    if (a.length === 0) return;
    if (Array.isArray(a[0])) {
        for (const sa of a as number[][]) flat(sa, xy, z);
    } else {
        if (a.length === 2) xy.push(...(a as number[]));
        else {
            xy.push(a[0], (a as number[])[1]);
            z.push((a as number[])[2]);
        }
    }
}

function deinterleaveZ(flatCoordinates: number[]): [number[], number[]] {
    const cLength = flatCoordinates.length / 3;
    const xy = new Array(cLength * 2);
    const z = new Array(cLength);
    for (let i = 0, j = 0; i < flatCoordinates.length; i += 3, j++) {
        xy[j * 2] = flatCoordinates[i];
        xy[j * 2 + 1] = flatCoordinates[i + 1];
        z[j] = flatCoordinates[i + 2];
    }
    return [xy, z];
}

function deinterleaveZM(flatCoordinates: number[]): [number[], number[], number[]] {
    const cLength = flatCoordinates.length / 4;
    const xy = new Array(cLength * 2);
    const z = new Array(cLength);
    const m = new Array(cLength);
    for (let i = 0, j = 0; i < flatCoordinates.length; i += 4, j++) {
        xy[j * 2] = flatCoordinates[i];
        xy[j * 2 + 1] = flatCoordinates[i + 1];
        z[j] = flatCoordinates[i + 2];
        m[j] = flatCoordinates[i + 3];
    }

    return [xy, z, m];
}

export function parseGeometry(geometry: ISimpleGeometry, headerGeomType: GeometryType): IParsedGeometry {
    let flatCoordinates: number[] | undefined;
    let xy: number[] | undefined;
    let z: number[] | undefined;
    let m: number[] | undefined;
    let ends: number[] | undefined;
    let parts: IParsedGeometry[] | undefined;

    let type = headerGeomType;
    if (type === GeometryType.Unknown) {
        type = toGeometryType(geometry.getType());
    }


    let flatEnds: number[] | undefined;
    if (type === GeometryType.MultiLineString) {
        if (geometry.getFlatCoordinates) flatCoordinates = geometry.getFlatCoordinates();
        flatEnds = (geometry as IMultiLineString).getEnds();
    } else if (type === GeometryType.Polygon) {
        if (geometry.getFlatCoordinates) flatCoordinates = geometry.getFlatCoordinates();
        flatEnds = (geometry as IPolygon).getEnds();
    } else if (type === GeometryType.MultiPolygon) {
        const mp = geometry as IMultiPolygon;
        parts = mp.getPolygons().map((p) => parseGeometry(p, GeometryType.Polygon));
    } else {
        if (geometry.getFlatCoordinates) flatCoordinates = geometry.getFlatCoordinates();
    }

    const layout = geometry.getLayout?.() ?? 'XY';
    if (flatCoordinates) {
        if (layout === 'XY') {
            xy = flatCoordinates;
        } else if (layout === 'XYZ') {
            [xy, z] = deinterleaveZ(flatCoordinates);
        } else if (layout === 'XYM') {
            [xy, m] = deinterleaveZ(flatCoordinates);
        } else if (layout === 'XYZM') {
            [xy, z, m] = deinterleaveZM(flatCoordinates);
        }
    }

    if (flatEnds) {
        let endDivision = 2;
        if (layout === 'XYZ' || layout === 'XYM') endDivision = 3;
        else if (layout === 'XYZM') endDivision = 4;
        ends = flatEnds.map((e) => e / endDivision);
    }

    return {
        xy,
        z,
        m,
        ends,
        type,
        parts,
    } as IParsedGeometry;
}

export function pairFlatCoordinates(xy: Float64Array, z?: Float64Array): number[][] {
    const newArray: number[][] = [];
    for (let i = 0; i < xy.length; i += 2) {
        const a = [xy[i], xy[i + 1]];
        if (z) a.push(z[i >> 1]);
        newArray.push(a);
    }
    return newArray;
}

export function toGeometryType(name?: string): GeometryType {
    if (!name) return GeometryType.Unknown;
    const type: GeometryType = (GeometryType as never)[name];
    return type;
}

import { GeometryType } from '../flat-geobuf/geometry-type.js';
import type { Geometry } from '../flat-geobuf/geometry.js';

import type { ISimpleGeometry } from '../generic/geometry.js';

import LineString from 'ol/geom/LineString.js';
import MultiLineString from 'ol/geom/MultiLineString.js';
import MultiPoint from 'ol/geom/MultiPoint.js';
import MultiPolygon from 'ol/geom/MultiPolygon.js';
import Point from 'ol/geom/Point.js';
import Polygon from 'ol/geom/Polygon.js';
import type { GeometryLayout } from 'ol/geom/Geometry.js';

function interleaveZ(flatCoordinates: number[], z: number[]): number[] {
    const newFlatCoordinates = new Array(flatCoordinates.length + z.length);
    for (let i = 0, j = 0, k = 0; i < flatCoordinates.length; i += 2, j++) {
        newFlatCoordinates[k++] = flatCoordinates[i];
        newFlatCoordinates[k++] = flatCoordinates[i + 1];
        newFlatCoordinates[k++] = z[j];
    }
    return newFlatCoordinates;
}

function interleaveZM(flatCoordinates: number[], z: number[], m: number[]): number[] {
    const newFlatCoordinates = new Array(flatCoordinates.length + z.length + m.length);
    for (let i = 0, j = 0, k = 0, l = 0; i < flatCoordinates.length; i += 2, j++) {
        newFlatCoordinates[k++] = flatCoordinates[i];
        newFlatCoordinates[k++] = flatCoordinates[i + 1];
        newFlatCoordinates[k++] = z[j];
        newFlatCoordinates[k++] = m[j];
    }
    return newFlatCoordinates;
}

export function createGeometry(geometry: Geometry | null, headerGeomType: GeometryType): ISimpleGeometry | undefined {
    let geomType: GeometryType | undefined;
    if (headerGeomType === GeometryType.Unknown) {
        geomType = geometry?.type();
    } else {
        geomType = headerGeomType;
    }

    if (!geometry) return;
    const xyArray = geometry.xyArray();
    if (xyArray) {
        let flatCoordinates = Array.from(xyArray);
        const z = geometry.zArray();
        const m = geometry.mArray();
        const ends = geometry.endsArray();
        let layout: GeometryLayout = 'XY';
        let endsMultiplier = 2;
        if (z && m) {
            layout = 'XYZM';
            flatCoordinates = interleaveZM(flatCoordinates, Array.from(z), Array.from(m));
            endsMultiplier = 4;
        } else if (z) {
            layout = 'XYZ';
            flatCoordinates = interleaveZ(flatCoordinates, Array.from(z));
            endsMultiplier = 3;
        } else if (m) {
            layout = 'XYM';
            flatCoordinates = interleaveZ(flatCoordinates, Array.from(m));
            endsMultiplier = 3;
        }
        const olEnds = ends ? Array.from(ends.map((e) => e * endsMultiplier)) : [flatCoordinates.length];
        switch (geomType) {
            case GeometryType.Point:
                return new Point(flatCoordinates);
            case GeometryType.MultiPoint:
                return new MultiPoint(flatCoordinates, layout);
            case GeometryType.LineString:
                return new LineString(flatCoordinates, layout);
            case GeometryType.MultiLineString:
                return new MultiLineString(flatCoordinates, layout, olEnds);
            case GeometryType.Polygon:
                return new Polygon(flatCoordinates, layout, olEnds);
        }
    }
    if (geomType === GeometryType.MultiPolygon) {
        const partsLength = geometry.partsLength();
        const polygons = new Array<Polygon>(partsLength);
        for (let i = 0; i < partsLength; i++) {
            polygons[i] = createGeometry(geometry.parts(i) as Geometry, GeometryType.Polygon) as Polygon;
        }
        return new MultiPolygon(polygons);
    }

    throw new Error('Unknown type');
}

import { GeometryType } from '../flat-geobuf/geometry-type.js';
import { Geometry } from '../flat-geobuf/geometry.js';

import { ISimpleGeometry } from '../generic/geometry.js';

import Point from 'ol/geom/Point';
import MultiPoint from 'ol/geom/MultiPoint';
import LineString from 'ol/geom/LineString';
import MultiLineString from 'ol/geom/MultiLineString';
import Polygon from 'ol/geom/Polygon';
import MultiPolygon from 'ol/geom/MultiPolygon';

export function createGeometryOl(
    geometry: Geometry | null,
    headerGeomType: GeometryType
): ISimpleGeometry | undefined {
    console.log('createGeometryOl with inferred type', headerGeomType);
    let geomType;
    if (headerGeomType === GeometryType.Unknown) {
        console.log('use per-feature geom type');
        console.log(geometry.type());
        geomType = geometry.type();
    } else {
        geomType = headerGeomType;
    }

    if (!geometry) return;
    const xyArray = geometry.xyArray();
    if (xyArray) {
        const xy = Array.from(geometry.xyArray() as ArrayLike<number>);
        const ends = geometry.endsArray();
        const olEnds = ends ? Array.from(ends.map((e) => e << 1)) : [xy.length];
        switch (geomType) {
            case GeometryType.Point:
                return new Point(xy);
            case GeometryType.MultiPoint:
                return new MultiPoint(xy, 'XY');
            case GeometryType.LineString:
                return new LineString(xy, 'XY');
            case GeometryType.MultiLineString:
                return new MultiLineString(xy, 'XY', olEnds);
            case GeometryType.Polygon:
                return new Polygon(xy, 'XY', olEnds);
        }
    } else if (geomType === GeometryType.MultiPolygon) {
        const mp = new MultiPolygon([]);
        for (let i = 0; i < geometry.partsLength(); i++)
            mp.appendPolygon(
                createGeometryOl(
                    geometry.parts(i) as Geometry,
                    GeometryType.Polygon
                ) as Polygon
            );
        return mp;
    }
    throw new Error('Unknown type');
}

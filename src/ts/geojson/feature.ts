import type { Feature as GeoJsonFeature } from 'geojson';
import type { Feature } from '../flat-geobuf/feature.js';
import type { Geometry } from '../flat-geobuf/geometry.js';
import { type IFeature, parseProperties } from '../generic/feature.js';
import type { HeaderMeta } from '../header-meta.js';
import { fromGeometry } from './geometry.js';

export interface IGeoJsonFeature extends IFeature, GeoJsonFeature {}

export function fromFeature(id: number, feature: Feature, header: HeaderMeta): IGeoJsonFeature {
    const columns = header.columns;
    const geometry = fromGeometry(feature.geometry() as Geometry, header.geometryType);
    const geoJsonfeature: GeoJsonFeature = {
        type: 'Feature',
        id,
        geometry,
        properties: parseProperties(feature, columns),
    };
    return geoJsonfeature;
}

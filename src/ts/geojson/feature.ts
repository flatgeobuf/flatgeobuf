import { Feature } from '../flat-geobuf/feature.js';
import { Geometry } from '../flat-geobuf/geometry.js';
import HeaderMeta from '../HeaderMeta.js';
import { fromGeometry, IGeoJsonGeometry } from './geometry.js';
import { parseProperties, IFeature } from '../generic/feature.js';

export interface IGeoJsonProperties {
    [key: string]: boolean | number | string | any;
}

export interface IGeoJsonFeature extends IFeature {
    type: string;
    geometry: IGeoJsonGeometry;
    properties?: IGeoJsonProperties;
}

export function fromFeature(
    feature: Feature,
    header: HeaderMeta
): IGeoJsonFeature {
    const columns = header.columns;
    const geometry = fromGeometry(feature.geometry() as Geometry);
    const geoJsonfeature: IGeoJsonFeature = {
        type: 'Feature',
        geometry,
    };
    if (columns && columns.length > 0)
        geoJsonfeature.properties = parseProperties(feature, columns);
    return geoJsonfeature;
}

import OlFeature from 'ol/Feature.js';

import type { Feature } from '../flat-geobuf/feature.js';
import { type IFeature, type IProperties, fromFeature as genericFromFeature } from '../generic/feature.js';
import type { ISimpleGeometry } from '../generic/geometry.js';
import type { HeaderMeta } from '../header-meta.js';
import { createGeometry } from './geometry.js';

function createFeature(id: number, geometry?: ISimpleGeometry, properties?: IProperties): IFeature {
    const olFeature = new OlFeature(geometry);
    olFeature.setId(id);
    const feature = olFeature as IFeature;
    if (properties && feature.setProperties) feature.setProperties(properties);
    return feature;
}

export function fromFeature(id: number, feature: Feature, header: HeaderMeta): IFeature {
    return genericFromFeature(id, feature, header, createGeometry, createFeature);
}

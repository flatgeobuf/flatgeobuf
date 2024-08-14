import OlFeature from 'ol/Feature.js';

import { Feature } from '../flat-geobuf/feature.js';
import type HeaderMeta from '../header-meta.js';
import { createGeometry } from './geometry.js';
import {
    fromFeature as genericFromFeature,
    type IFeature,
} from '../generic/feature.js';
import { type ISimpleGeometry } from '../generic/geometry.js';

function createFeature(
    geometry?: ISimpleGeometry,
    properties?: Record<string, unknown>,
): IFeature {
    const feature = new OlFeature(geometry) as IFeature;
    if (properties && feature.setProperties) feature.setProperties(properties);
    return feature;
}

export function fromFeature(feature: Feature, header: HeaderMeta): IFeature {
    return genericFromFeature(feature, header, createGeometry, createFeature);
}

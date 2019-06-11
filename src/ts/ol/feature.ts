import { Feature } from '../feature_generated'
import HeaderMeta from '../HeaderMeta'
import { createGeometry } from './geometry'
import { fromFeature as genericFromFeature, IFeature } from '../generic/feature'
import { ISimpleGeometry } from '../generic/geometry'

import OLFeature from 'ol/Feature'

export function createFeature(geometry: ISimpleGeometry, properties: any): IFeature {
    const olFeature = new OLFeature(geometry)
    if (properties)
        olFeature.setProperties(properties)
    return olFeature
}

export function fromFeature(feature: Feature, header: HeaderMeta): IFeature {
    return genericFromFeature(feature, header, createGeometry, createFeature)
}
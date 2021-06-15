import { default as OLFeature } from 'ol/Feature'

import { GeometryType } from '../flat-geobuf/geometry-type'
import { Feature } from '../flat-geobuf/feature'
import { Geometry } from '../flat-geobuf/geometry'
import HeaderMeta from '../HeaderMeta'
import { createGeometryOl } from './geometry'
import { fromFeature as genericFromFeature, IFeature } from '../generic/feature'
import { ISimpleGeometry } from '../generic/geometry'

export function createFeatureOl(geometry?: ISimpleGeometry, properties?: Record<string, unknown>): IFeature {
    const feature = new OLFeature(geometry) as IFeature
    if (properties && feature.setProperties)
        feature.setProperties(properties)
    return feature
}

export function fromFeature(feature: Feature, header: HeaderMeta): IFeature {
    function createFeature(geometry?: ISimpleGeometry, properties?: Record<string, unknown>) {
        return createFeatureOl(geometry, properties)
    }
    function createGeometry(geometry: Geometry | null, type: GeometryType) {
        return createGeometryOl(geometry, type)
    }
    return genericFromFeature(feature, header, createGeometry, createFeature)
}

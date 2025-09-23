import OlFeature from 'ol/Feature.js';
import { transformGeometryWithOptions } from 'ol/format/Feature.js';
import RenderFeature, { type Type } from 'ol/render/Feature.js';
import type { Feature } from '../flat-geobuf/feature.js';
import {
    fromFeature as genericFromFeature,
    type ICreateFeature,
    type IFeature,
    type IProperties,
} from '../generic/feature.js';
import type { FromFeatureFn } from '../generic/featurecollection';
import type { ISimpleGeometry } from '../generic/geometry.js';
import type { HeaderMeta } from '../header-meta.js';
import { createGeometry } from './geometry.js';

function getCreateFeatureFn(dataProjection: string, featureProjection?: string): ICreateFeature {
    return function createFeature(id: number, geometry?: ISimpleGeometry, properties?: IProperties): IFeature {
        const olFeature = new OlFeature(geometry);
        if (featureProjection && dataProjection !== featureProjection) {
            olFeature.getGeometry()?.transform(dataProjection, featureProjection);
        }
        olFeature.setId(id);
        const feature = olFeature as IFeature;
        if (properties && feature.setProperties) feature.setProperties(properties);
        return feature;
    };
}

function getCreateRenderFeatureFn(dataProjection: string, featureProjection?: string): ICreateFeature {
    return function createRenderFeature(id: number, geometry?: ISimpleGeometry, properties?: IProperties): IFeature {
        const geometryType = geometry?.getType() === 'MultiPolygon' ? 'Polygon' : geometry?.getType();
        if (geometryType === 'GeometryCollection' || geometryType === 'Circle') {
            throw new Error(`Unsupported geometry type: ${geometryType}`);
        }
        const flatCoordinates = geometry?.getFlatCoordinates?.();
        const stride = geometry?.getLayout?.().length;
        if (!flatCoordinates || !stride) {
            throw new Error(`Geometry without coordinates: ${geometry}`);
        }
        const renderFeature = new RenderFeature(
            geometryType as Type,
            flatCoordinates,
            [flatCoordinates.length],
            stride,
            properties || {},
            id,
        );
        return transformGeometryWithOptions(renderFeature.enableSimplifyTransformed(), false, {
            dataProjection,
            featureProjection,
        }) as IFeature;
    };
}

export function getFromFeatureFn(
    classType: typeof OlFeature | typeof RenderFeature = OlFeature,
    dataProjection = 'EPSG:4326',
    featureProjection?: string,
): FromFeatureFn {
    let createFeatureFn: ICreateFeature;
    if (classType === RenderFeature) {
        createFeatureFn = getCreateRenderFeatureFn(dataProjection, featureProjection);
    } else {
        createFeatureFn = getCreateFeatureFn(dataProjection, featureProjection);
    }
    return (id: number, feature: Feature, header: HeaderMeta) =>
        genericFromFeature(id, feature, header, createGeometry, createFeatureFn);
}

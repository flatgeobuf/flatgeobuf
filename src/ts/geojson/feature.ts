import { Feature } from '../feature_generated'
import HeaderMeta from '../HeaderMeta'
import { fromGeometry, IGeoJsonGeometry } from './geometry'
import { parseProperties, IFeature } from '../generic/feature'

export interface IGeoJsonProperties {
    [key: string]: boolean | number | string | any
}

export interface IGeoJsonFeature extends IFeature {
    type: string
    geometry: IGeoJsonGeometry
    properties?: IGeoJsonProperties
}

export function fromFeature(feature: Feature, header: HeaderMeta): IGeoJsonFeature {
    const columns = header.columns
    const geometry = fromGeometry(feature.geometry(), header.geometryType)
    const properties = parseProperties(feature, columns)
    const geoJsonfeature: IGeoJsonFeature = {
        type: 'Feature',
        geometry
    }
    if (properties)
        geoJsonfeature.properties = properties
    return geoJsonfeature
}

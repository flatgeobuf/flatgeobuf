import ColumnMeta from '../ColumnMeta'
import { ColumnType } from '../header_generated'
import HeaderMeta from '../HeaderMeta'

import { fromFeature, IGeoJsonFeature } from './feature'
import { parseGeometry } from './geometry'
import {
    buildHeader,
    deserialize as genericDeserialize,
    deserializeStream as genericDeserializeStream,
    deserializeFiltered as genericDeserializeFiltered } from '../generic/featurecollection'
import { toGeometryType } from '../generic/geometry'
import { Rect } from '../packedrtree'
import { buildFeature, IProperties } from '../generic/feature'
import { HeaderMetaFn } from '../generic'
import { magicbytes } from '../constants'

export interface IGeoJsonFeatureCollection {
    type: string,
    features: IGeoJsonFeature[]
}

export function serialize(featurecollection: IGeoJsonFeatureCollection): Uint8Array {
    const headerMeta = introspectHeaderMeta(featurecollection)
    const header = buildHeader(headerMeta)
    const features: Uint8Array[] = featurecollection.features
        .map(f => buildFeature(parseGeometry(f.geometry), f.properties as IProperties, headerMeta))
    const featuresLength = features
        .map(f => f.length)
        .reduce((a, b) => a + b)
    const uint8 = new Uint8Array(magicbytes.length + header.length + featuresLength)
    uint8.set(header, magicbytes.length)
    let offset = magicbytes.length + header.length
    for (const feature of features) {
        uint8.set(feature, offset)
        offset += feature.length
    }
    uint8.set(magicbytes)
    return uint8
}

export function deserialize(bytes: Uint8Array, headerMetaFn?: HeaderMetaFn): IGeoJsonFeatureCollection {
    const features = genericDeserialize(bytes, fromFeature, headerMetaFn)
    return {
        type: 'FeatureCollection',
        features,
    } as IGeoJsonFeatureCollection
}

export function deserializeStream(stream: ReadableStream, headerMetaFn?: HeaderMetaFn): AsyncGenerator<any, void, unknown> {
    return genericDeserializeStream(stream, fromFeature, headerMetaFn)
}

export function deserializeFiltered(url: string, rect: Rect, headerMetaFn?: HeaderMetaFn): AsyncGenerator<any, void, unknown> {
    return genericDeserializeFiltered(url, rect, fromFeature, headerMetaFn)
}

function valueToType(value: boolean | number | string | unknown): ColumnType {
    if (typeof value === 'boolean')
        return ColumnType.Bool
    else if (typeof value === 'number')
        if (value % 1 === 0)
            return ColumnType.Int
        else
            return ColumnType.Double
    else if (typeof value === 'string')
        return ColumnType.String
    else if (value === null)
        return ColumnType.String
    else
        throw new Error(`Unknown type (value '${value}')`)
}

function introspectHeaderMeta(featurecollection: IGeoJsonFeatureCollection) : HeaderMeta {
    const feature = featurecollection.features[0]
    const properties = feature.properties

    let columns: ColumnMeta[] | null = null
    if (properties)
        columns = Object.keys(properties)
            .map(k => new ColumnMeta(k, valueToType(properties[k]), null, null, -1, -1, -1, true, false, false))

    const headerMeta = new HeaderMeta(toGeometryType(feature.geometry.type), columns, featurecollection.features.length, 0, null, null, null, null)

    return headerMeta
}

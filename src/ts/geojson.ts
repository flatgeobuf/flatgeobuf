import {
    IGeoJsonFeatureCollection,
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize } from './geojson/featurecollection'

import { Rect } from './packedrtree'
import { IGeoJsonFeature } from './geojson/feature'
import { HeaderMetaFn } from './generic'

/**
 * @param geojson GeoJSON object to serialize
 */
export function serialize(geojson: IGeoJsonFeatureCollection): Uint8Array {
    const bytes = fcSerialize(geojson)
    return bytes
}

/**
 * 
 * @param input Input byte array, stream or string
 * @param rect Filter rectangle
 * @param headerMetaFn Callback that will recieve [[HeaderMeta]] when available
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn) :
    IGeoJsonFeatureCollection | AsyncGenerator<IGeoJsonFeature>
{
    if (input instanceof Uint8Array)
        return fcDeserialize(input, headerMetaFn)
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input, headerMetaFn)
    else
        return fcDeserializeFiltered(input, rect, headerMetaFn)
}
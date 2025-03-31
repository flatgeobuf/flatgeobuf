import {
    deserialize as fcDeserialize,
    deserializeFiltered as fcDeserializeFiltered,
    deserializeStream as fcDeserializeStream,
    serialize as fcSerialize,
} from './geojson/featurecollection.js';

import type { FeatureCollection as GeoJsonFeatureCollection } from 'geojson';

import type { HeaderMetaFn } from './generic.js';
import type { IGeoJsonFeature } from './geojson/feature.js';
import type { Rect } from './packedrtree.js';

/**
 * Serialize GeoJSON to FlatGeobuf
 * @param geojson GeoJSON object to serialize
 */
export function serialize(geojson: GeoJsonFeatureCollection, crsCode = 0): Uint8Array {
    const bytes = fcSerialize(geojson, crsCode);
    return bytes;
}

/**
 * Deserialize FlatGeobuf into GeoJSON features
 * @param url Input string
 * @param rect Filter rectangle - NOT USED
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    url: string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache?: boolean,
): AsyncGenerator<IGeoJsonFeature>;

/**
 * Deserialize FlatGeobuf from a typed array into GeoJSON features
 * NOTE: Does not support spatial filtering
 * @param typedArray Input byte array
 * @param rect Filter rectangle - NOT USED
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    typedArray: Uint8Array,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache?: boolean,
): GeoJsonFeatureCollection;

/**
 * Deserialize FlatGeobuf from a stream into GeoJSON features
 * NOTE: Does not support spatial filtering
 * @param stream stream
 * @param rect Filter rectangle
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    stream: ReadableStream,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache?: boolean,
): AsyncGenerator<IGeoJsonFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
): GeoJsonFeatureCollection | AsyncGenerator<IGeoJsonFeature> {
    if (input instanceof Uint8Array) return fcDeserialize(input, rect, headerMetaFn);
    if (input instanceof ReadableStream)
        return fcDeserializeStream(input, headerMetaFn) as AsyncGenerator<IGeoJsonFeature>;
    return fcDeserializeFiltered(input, rect!, headerMetaFn, nocache) as AsyncGenerator<IGeoJsonFeature>;
}

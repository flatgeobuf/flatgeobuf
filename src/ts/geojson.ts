import {
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize,
} from './geojson/featurecollection.js';

import { FeatureCollection as GeoJsonFeatureCollection } from 'geojson';

import { Rect } from './packedrtree.js';
import { IGeoJsonFeature } from './geojson/feature.js';
import { HeaderMetaFn } from './generic.js';

/**
 * Serialize GeoJSON to FlatGeobuf
 * @param geojson GeoJSON object to serialize
 */
export function serialize(geojson: GeoJsonFeatureCollection): Uint8Array {
    const bytes = fcSerialize(geojson);
    return bytes;
}

/**
 * Deserialize FlatGeobuf into GeoJSON features
 * @param url Input string
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    url: string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IGeoJsonFeature, any, unknown>;

/**
 * Deserialize FlatGeobuf from a typed array into GeoJSON features
 * @note Does not support spatial filtering
 * @param typedArray Input byte array
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    typedArray: Uint8Array,
    headerMetaFn?: HeaderMetaFn,
): GeoJsonFeatureCollection;

/**
 * Deserialize FlatGeobuf from a stream into GeoJSON features
 * @note Does not support spatial filtering
 * @param stream stream
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    stream: ReadableStream,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IGeoJsonFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rectOrFn?: Rect | HeaderMetaFn,
    headerMetaFn?: HeaderMetaFn,
): GeoJsonFeatureCollection | AsyncGenerator<IGeoJsonFeature> {
    if (input instanceof Uint8Array)
        return fcDeserialize(input, rectOrFn as HeaderMetaFn);
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input, rectOrFn as HeaderMetaFn);
    else return fcDeserializeFiltered(input, rectOrFn as Rect, headerMetaFn);
}

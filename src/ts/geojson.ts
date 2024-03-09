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
export function serialize(geojson: GeoJsonFeatureCollection, crsCode: number = 0): Uint8Array {
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
): AsyncGenerator<IGeoJsonFeature, any, unknown>;

/**
 * Deserialize FlatGeobuf from a typed array into GeoJSON features
 * @note Does not support spatial filtering
 * @param typedArray Input byte array
 * @param rect Filter rectangle - NOT USED
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    typedArray: Uint8Array,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): GeoJsonFeatureCollection;

/**
 * Deserialize FlatGeobuf from a stream into GeoJSON features
 * @note Does not support spatial filtering
 * @param stream stream
 * @param rect Filter rectangle
 * @param headerMetaFn Callback that will receive header metadata when available
 */
export function deserialize(
    stream: ReadableStream,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IGeoJsonFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): GeoJsonFeatureCollection | AsyncGenerator<IGeoJsonFeature> {
    if (input instanceof Uint8Array) return fcDeserialize(input, headerMetaFn);
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input, headerMetaFn);
    else return fcDeserializeFiltered(input, rect!, headerMetaFn);
}

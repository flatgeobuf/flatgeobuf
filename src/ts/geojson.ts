import type { FeatureCollection as GeoJsonFeatureCollection } from 'geojson';
import type { IGeoJsonFeature } from './geojson/feature.js';
import type { DeserializeOptions } from './generic/deserialize.js';
export type { IGeoJsonFeature } from './geojson/feature.js';
export type { DeserializeOptions } from './generic/deserialize.js';
import {
    deserialize as fcDeserialize,
    deserializeFiltered as fcDeserializeFiltered,
    deserializeStream as fcDeserializeStream,
    serialize as fcSerialize,
} from './geojson/featurecollection.js';

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
 * @param inputOrOptions Input byte array, stream or URL string, or deserializer options including input
 */
export function deserialize(
    inputOrOptions: Uint8Array | ReadableStream | string | DeserializeOptions,
): AsyncGenerator<IGeoJsonFeature> {
    const options: DeserializeOptions = inputOrOptions instanceof Uint8Array || inputOrOptions instanceof ReadableStream || typeof inputOrOptions === 'string'
        ? { input: inputOrOptions }
        : inputOrOptions;
    const { input } = options;
    if (input instanceof Uint8Array) return fcDeserialize(options) as AsyncGenerator<IGeoJsonFeature>;
    if (input instanceof ReadableStream) return fcDeserializeStream(options) as AsyncGenerator<IGeoJsonFeature>;
    if (typeof input === 'string') return fcDeserializeFiltered(options) as AsyncGenerator<IGeoJsonFeature>;
    throw new Error('Invalid input type');
}

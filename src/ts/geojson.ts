import type { FeatureCollection as GeoJsonFeatureCollection } from 'geojson';
import type { DeserializeOptions } from './generic/deserialize.js';
import type { IGeoJsonFeature } from './geojson/feature.js';

export type { DeserializeOptions } from './generic/deserialize.js';
export type { IGeoJsonFeature } from './geojson/feature.js';

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
 * @param input Input byte array, stream or URL string
 * @param options Optional deserializer options (rect, headerMetaFn, headers, nocache)
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    options?: DeserializeOptions,
): AsyncGenerator<IGeoJsonFeature> {
    if (input instanceof Uint8Array) return fcDeserialize(input, options) as AsyncGenerator<IGeoJsonFeature>;
    if (input instanceof ReadableStream) return fcDeserializeStream(input, options) as AsyncGenerator<IGeoJsonFeature>;
    if (typeof input === 'string') return fcDeserializeFiltered(input, options) as AsyncGenerator<IGeoJsonFeature>;
    throw new Error('Invalid input type');
}

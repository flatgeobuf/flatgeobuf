import type { FeatureCollection as GeoJsonFeatureCollection } from 'geojson';
import type { DeserializeOptions } from './generic/deserialize.js';
import { deserialize as genericDeserialize } from './generic.js';
import { fromFeature, type IGeoJsonFeature } from './geojson/feature.js';

import { serialize as fcSerialize } from './geojson/featurecollection.js';

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
    return genericDeserialize(input, { ...options, fromFeature }) as AsyncGenerator<IGeoJsonFeature>;
}

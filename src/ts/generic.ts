import type { DeserializeContext } from './generic/deserialize.js';
import type { IFeature } from './generic/feature.js';
import {
    deserialize as deserializeArray,
    deserializeFiltered,
    deserializeStream,
    readMetadata as readMetadataUrl,
} from './generic/featurecollection.js';
import type { HeaderMeta } from './header-meta.js';

export { ColumnType } from './flat-geobuf/column-type.js';
export { GeometryType } from './flat-geobuf/geometry-type.js';

/** Callback function for receiving header metadata */
export type HeaderMetaFn = (headerMeta: HeaderMeta) => void;

/**
 * Deserialize FlatGeobuf into generic features.
 * Streams are not supporting spatial filtering.
 * @param input Input byte array, stream or URL string
 * @param ctx Deserialize context with fromFeature callback
 */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    ctx: DeserializeContext,
): AsyncGenerator<IFeature> {
    if (input instanceof Uint8Array) return deserializeArray(input, ctx);
    if (input instanceof ReadableStream) return deserializeStream(input, ctx);
    return deserializeFiltered(input, ctx);
}
/**
 * read only Metadata from a remote FlatGeobuf file
 * @param url Input string
 * @param nocache Disable caching
 * @param headers Additional HTTP headers
 */
export function readMetadata(url: string, nocache = false, headers: HeadersInit = {}): Promise<HeaderMeta> {
    //TODO: support reading from typed array or stream
    return readMetadataUrl(url, nocache, headers);
}

export type { DeserializeContext, DeserializeOptions } from './generic/deserialize.js';
export type { IFeature, IProperties } from './generic/feature.js';
export { parseProperties } from './generic/feature.js';
export { type FromFeatureFn, serialize } from './generic/featurecollection.js';
export type { ISimpleGeometry } from './generic/geometry.js';

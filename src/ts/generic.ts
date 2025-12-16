import type { IFeature } from './generic/feature.js';
import {
    deserialize as deserializeArray,
    deserializeFiltered,
    deserializeStream,
    type FromFeatureFn,
    readMetadata as readMetadataUrl,
} from './generic/featurecollection.js';
import type { HeaderMeta } from './header-meta.js';
import type { Rect } from './packedrtree.js';

export { ColumnType } from './flat-geobuf/column-type.js';
export { GeometryType } from './flat-geobuf/geometry-type.js';

/** Callback function for receiving header metadata */
export type HeaderMetaFn = (headerMeta: HeaderMeta) => void;

/**
 * Deserialize FlatGeobuf from a URL into generic features
 * @param url Input string
 * @param fromFeature Callback that receives generic features
 * @param rect Filter rectangle
 */
export function deserialize(url: string, fromFeature: FromFeatureFn, rect?: Rect): AsyncGenerator<IFeature>;

/**
 * Deserialize FlatGeobuf from a typed array into generic features
 * @param typedArray Input byte array
 * @param fromFeature Callback that receives generic features
 */
export function deserialize(typedArray: Uint8Array, fromFeature: FromFeatureFn, rect?: Rect): IFeature[];

/**
 * Deserialize FlatGeobuf from a stream into generic features
 * NOTE: Does not support spatial filtering
 * @param input stream
 * @param fromFeature Callback that receives generic features
 */
export function deserialize(input: ReadableStream, fromFeature: FromFeatureFn): AsyncGenerator<IFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    fromFeature: FromFeatureFn,
    rect?: Rect,
    nocache = false,
    headers: HeadersInit = {},
): IFeature[] | AsyncGenerator<IFeature> {
    if (input instanceof Uint8Array) return deserializeArray(input, fromFeature, rect);
    if (input instanceof ReadableStream) return deserializeStream(input, fromFeature);
    return deserializeFiltered(input, rect as Rect, fromFeature, undefined, nocache, headers);
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

export { parseProperties } from './generic/feature.js';
export { serialize } from './generic/featurecollection.js';

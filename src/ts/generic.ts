import {
    deserialize as deserializeArray,
    deserializeStream,
    deserializeFiltered,
    FromFeatureFn,
} from './generic/featurecollection.js';

import { Rect } from './packedrtree.js';
import { IFeature } from './generic/feature.js';
import HeaderMeta from './header-meta.js';

export { GeometryType } from './flat-geobuf/geometry-type.js';
export { ColumnType } from './flat-geobuf/column-type.js';

/** Callback function for receiving header metadata */
export type HeaderMetaFn = (headerMeta: HeaderMeta) => void;

/**
 * Deserialize FlatGeobuf into generic features
 * @param input Input byte array
 * @param fromFeature Callback that receives generic features
 * @param rect Filter rectangle
 */
export function deserialize(
    input: Uint8Array,
    fromFeature: FromFeatureFn,
    rect?: Rect,
): IFeature[];

/**
 * Deserialize FlatGeobuf into generic features
 * @param input Input string
 * @param fromFeature Callback that receives generic features
 * @param rect Filter rectangle
 */
export function deserialize(
    input: string,
    fromFeature: FromFeatureFn,
    rect?: Rect,
): AsyncGenerator<IFeature, any, unknown>;

/**
 * Deserialize FlatGeobuf into generic features
 * @param input stream
 * @param fromFeature Callback that receives generic features
 * @param rect Filter rectangle
 */
export function deserialize(
    input: ReadableStream,
    fromFeature: FromFeatureFn,
    rect?: Rect,
): AsyncGenerator<IFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    fromFeature: FromFeatureFn,
    rect?: Rect,
): any[] | AsyncGenerator<IFeature> {
    if (input instanceof Uint8Array)
        return deserializeArray(input, fromFeature);
    else if (input instanceof ReadableStream)
        return deserializeStream(input, fromFeature);
    else return deserializeFiltered(input, rect as Rect, fromFeature);
}

export { serialize } from './generic/featurecollection.js';

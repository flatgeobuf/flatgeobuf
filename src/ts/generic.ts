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
 * Deserialize FlatGeobuf from a URL into generic features
 * @note Supports spatial filtering
 * @param input Input string
 * @param fromFeature Callback that receives generic features
 * @param rect Filter rectangle
 */
export function deserialize(
    url: string,
    fromFeature: FromFeatureFn,
    rect?: Rect,
): AsyncGenerator<IFeature, any, unknown>;

/**
 * Deserialize FlatGeobuf from a typed array into generic features
 * @note Does not support spatial filtering
 * @param typedArray Input byte array
 * @param fromFeature Callback that receives generic features
 */
export function deserialize(
    typedArray: Uint8Array,
    fromFeature: FromFeatureFn,
): IFeature[];

/**
 * Deserialize FlatGeobuf from a stream into generic features
 * @note Does not support spatial filtering
 * @param stream stream
 * @param fromFeature Callback that receives generic features
 */
export function deserialize(
    input: ReadableStream,
    fromFeature: FromFeatureFn,
): AsyncGenerator<IFeature>;

/** Implementation */
export function deserialize(
    input: Uint8Array | ReadableStream | string,
    fromFeature: FromFeatureFn,
    rect?: Rect,
    nocache: boolean = false,
): IFeature[] | AsyncGenerator<IFeature> {
    if (input instanceof Uint8Array)
        return deserializeArray(input, fromFeature);
    else if (input instanceof ReadableStream)
        return deserializeStream(input, fromFeature);
    else
        return deserializeFiltered(
            input,
            rect as Rect,
            fromFeature,
            undefined,
            nocache,
        );
}

export { serialize } from './generic/featurecollection.js';

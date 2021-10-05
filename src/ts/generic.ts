import {
    deserialize as deserializeArray,
    deserializeStream,
    deserializeFiltered,
    FromFeatureFn,
} from './generic/featurecollection';

import { Rect } from './packedrtree.js';
import { IFeature } from './generic/feature.js';
import HeaderMeta from './HeaderMeta.js';

export type HeaderMetaFn = (headerMeta: HeaderMeta) => void;

export function deserialize(
    input: Uint8Array | ReadableStream | string,
    fromFeature: FromFeatureFn,
    rect?: Rect
): any[] | AsyncGenerator<IFeature> {
    if (input instanceof Uint8Array)
        return deserializeArray(input, fromFeature);
    else if (input instanceof ReadableStream)
        return deserializeStream(input, fromFeature);
    else return deserializeFiltered(input, rect as Rect, fromFeature);
}

export { serialize } from './generic/featurecollection';

export { GeometryType } from './flat-geobuf/geometry-type';
export { ColumnType } from './flat-geobuf/column-type';

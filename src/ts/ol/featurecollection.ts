import type { IFeature } from '../generic/feature.js';
import {
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize,
} from '../generic/featurecollection.js';
import type { HeaderMetaFn } from '../generic.js';
import type { Rect } from '../packedrtree.js';
import { fromFeature } from './feature.js';

export { serialize };

export async function* deserialize(
    bytes: Uint8Array,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IFeature> {
    yield* genericDeserialize(bytes, fromFeature, rect, headerMetaFn);
}

export function deserializeStream(stream: ReadableStream, headerMetaFn?: HeaderMetaFn): AsyncGenerator<IFeature> {
    return genericDeserializeStream(stream, fromFeature, headerMetaFn);
}

export function deserializeFiltered(
    url: string,
    rect: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
    headers: HeadersInit = {},
): AsyncGenerator<IFeature> {
    return genericDeserializeFiltered(url, rect, fromFeature, headerMetaFn, nocache, headers);
}

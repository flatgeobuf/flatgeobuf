import type { HeaderMetaFn } from '../generic.js';
import type { IFeature } from '../generic/feature.js';
import {
    deserialize as genericDeserialize,
    deserializeFiltered as genericDeserializeFiltered,
    deserializeStream as genericDeserializeStream,
    serialize,
} from '../generic/featurecollection.js';
import type { Rect } from '../packedrtree.js';
import { fromFeature } from './feature.js';

export { serialize };

export function deserialize(bytes: Uint8Array, headerMetaFn?: HeaderMetaFn): IFeature[] {
    return genericDeserialize(bytes, fromFeature, headerMetaFn);
}

export function deserializeStream(stream: ReadableStream, headerMetaFn?: HeaderMetaFn): AsyncGenerator<IFeature> {
    return genericDeserializeStream(stream, fromFeature, headerMetaFn);
}

export function deserializeFiltered(
    url: string,
    rect: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
): AsyncGenerator<IFeature> {
    return genericDeserializeFiltered(url, rect, fromFeature, headerMetaFn, nocache);
}

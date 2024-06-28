import {
    serialize,
    deserialize as genericDeserialize,
    deserializeStream as genericDeserializeStream,
    deserializeFiltered as genericDeserializeFiltered,
} from '../generic/featurecollection.js';
import { type IFeature } from '../generic/feature.js';
import { fromFeature } from './feature.js';
import { type HeaderMetaFn } from '../generic.js';
import { type Rect } from '../packedrtree.js';

export { serialize as serialize };

export function deserialize(
    bytes: Uint8Array,
    headerMetaFn?: HeaderMetaFn,
): IFeature[] {
    return genericDeserialize(bytes, fromFeature, headerMetaFn);
}

export function deserializeStream(
    stream: ReadableStream,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<any, void, unknown> {
    return genericDeserializeStream(stream, fromFeature, headerMetaFn);
}

export function deserializeFiltered(
    url: string,
    rect: Rect,
    headerMetaFn?: HeaderMetaFn,
    nocache: boolean = false,
): AsyncGenerator<any, void, unknown> {
    return genericDeserializeFiltered(
        url,
        rect,
        fromFeature,
        headerMetaFn,
        nocache,
    );
}

import {
    serialize,
    deserialize as genericDeserialize,
    deserializeStream as genericDeserializeStream } from '../generic/featurecollection'
import { IFeature } from '../generic/feature'
import { fromFeature } from './feature'
import { HeaderMetaFn } from '../generic'

export { serialize as serialize }

export function deserialize(bytes: Uint8Array, headerMetaFn?: HeaderMetaFn): IFeature[] {
    return genericDeserialize(bytes, (f, h) => fromFeature(f, h), headerMetaFn)
}

export function deserializeStream(stream: ReadableStream, headerMetaFn?: HeaderMetaFn): AsyncGenerator<any, void, unknown> {
    return genericDeserializeStream(stream, (f, h) => fromFeature(f, h), headerMetaFn)
}
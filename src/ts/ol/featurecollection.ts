import {
    serialize,
    deserialize as genericDeserialize,
    deserializeStream as genericDeserializeStream } from '../generic/featurecollection'
import { IFeature } from '../generic/feature'
import { fromFeature } from './feature'

export { serialize as serialize }

export function deserialize(bytes: Uint8Array, ol: any): IFeature[] {
    return genericDeserialize(bytes, (f, h) => fromFeature(f, h, ol))
}

export function deserializeStream(stream: any, ol: any) {
    return genericDeserializeStream(stream, (f, h) => fromFeature(f, h, ol))
}
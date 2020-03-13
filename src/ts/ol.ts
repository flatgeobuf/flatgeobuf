import { ReadableStream } from 'web-streams-polyfill/ponyfill'

import { IFeature } from './generic/feature'

import { 
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    serialize as fcSerialize
} from './ol/featurecollection'

export function serialize(features: IFeature[]) {
    const bytes = fcSerialize(features)
    return bytes
}

export function deserializeStream(stream: ReadableStream) {
    return fcDeserializeStream(stream)
}

export function deserialize(bytes: Uint8Array) {
    const features = fcDeserialize(bytes)
    return features
}

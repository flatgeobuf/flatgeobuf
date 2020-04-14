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

export function deserialize(input: Uint8Array | ReadableStream) {
    if (input instanceof ReadableStream)
        return fcDeserializeStream(input)
    else
        return fcDeserialize(input)
}
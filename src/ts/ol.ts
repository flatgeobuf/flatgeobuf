import { IFeature } from './generic/feature'

import {
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    serialize as fcSerialize
} from './ol/featurecollection'
import { HeaderMetaFn } from './generic'

export function serialize(features: IFeature[]): Uint8Array {
    const bytes = fcSerialize(features)
    return bytes
}

export function deserialize(input: Uint8Array | ReadableStream, headerMetaFn?: HeaderMetaFn)
    : AsyncGenerator<IFeature> | IFeature[]
{
    if (input instanceof ReadableStream)
        return fcDeserializeStream(input, headerMetaFn)
    else
        return fcDeserialize(input, headerMetaFn)
}
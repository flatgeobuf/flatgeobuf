import { ReadableStream } from 'web-streams-polyfill/ponyfill'

import {
    IGeoJsonFeatureCollection,
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize } from './geojson/featurecollection'

import { Rect } from './packedrtree'

export function serialize(geojson: any) {
    const bytes = fcSerialize(geojson)
    return bytes
}

export function deserialize(input: Uint8Array | ReadableStream | string, rect?: Rect) :
    IGeoJsonFeatureCollection | AsyncGenerator {
    if (input instanceof Uint8Array)
        return fcDeserialize(input)
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input)
    else
        return fcDeserializeFiltered(input, rect)
}
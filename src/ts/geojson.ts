import { ReadableStream } from 'web-streams-polyfill/ponyfill'

import {
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize } from './geojson/featurecollection'

import { Rect } from './packedrtree'

export function serialize(geojson: any) {
    const bytes = fcSerialize(geojson)
    return bytes
}

export function deserialize(bytes: Uint8Array) {
    const geojson = fcDeserialize(bytes)
    return geojson
}

export function deserializeStream(stream: ReadableStream) {
    const generator = fcDeserializeStream(stream)
    return generator
}

export function deserializeFiltered(url, rect: Rect) {
    const generator = fcDeserializeFiltered(url, rect)
    return generator
}
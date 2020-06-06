import {
    IGeoJsonFeatureCollection,
    deserialize as fcDeserialize,
    deserializeStream as fcDeserializeStream,
    deserializeFiltered as fcDeserializeFiltered,
    serialize as fcSerialize } from './geojson/featurecollection'

import { Rect } from './packedrtree'
import { IGeoJsonFeature } from './geojson/feature'

export function serialize(geojson: IGeoJsonFeatureCollection): Uint8Array {
    const bytes = fcSerialize(geojson)
    return bytes
}

export function deserialize(input: Uint8Array | ReadableStream | string, rect?: Rect) :
    IGeoJsonFeatureCollection | AsyncGenerator<IGeoJsonFeature> {
    if (input instanceof Uint8Array)
        return fcDeserialize(input)
    else if (input instanceof ReadableStream)
        return fcDeserializeStream(input)
    else
        return fcDeserializeFiltered(input, rect)
}
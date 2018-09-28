import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from './flatgeobuf_generated'

import { deserialize as fcDeserialize, serialize as fcSerialize } from './geojson/featurecollection'

export function serialize(geojson: any) {
    const bytes = fcSerialize(geojson)
    return bytes
}

export function deserialize(bytes: Uint8Array) {
    const geojson = fcDeserialize(bytes)
    return geojson
}

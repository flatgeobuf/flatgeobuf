import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from './flatgeobuf_generated'

import { fromFlatGeobuf, toFlatGeobuf } from './geojson/featurecollection'

export function fromGeoJson(geojson: any) {
    const bytes = toFlatGeobuf(geojson)
    return bytes
}

export function toGeoJson(bytes: Uint8Array) {
    const geojson = fromFlatGeobuf(bytes)
    return geojson
}

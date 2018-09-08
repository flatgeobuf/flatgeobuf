import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from './flatgeobuf'

import { buildGeometry } from './geojson/geometry'

export function fromGeoJson(geojson: any) {
    const builder = new flatbuffers.Builder(0)

    buildGeometry(builder, geojson)

    return builder.dataBuffer()
}

export function toGeoJson(buf: flatbuffers.ByteBuffer) {
    const geometry = FlatGeobuf.Geometry.getRootAsGeometry(buf)
    const type = geometry.type()
    return type
}

import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

import { buildGeometry, fromGeometry } from './geometry'

const Feature = FlatGeobuf.Feature

export function buildFeature(feature: any) {
    const builder = new flatbuffers.Builder(0)
    const geometryOffset = buildGeometry(builder, feature.geometry)
    Feature.startFeature(builder)
    Feature.addGeometry(builder, geometryOffset)
    const offset = Feature.endFeature(builder)
    builder.finish(offset)
    return builder.dataBuffer()
}

export function fromFeature(feature: FlatGeobuf.Feature) {
    const geometry = fromGeometry(feature.geometry())
    return {
        type: 'Feature',
        geometry,
    }
}

import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'
import LayerMeta from '../LayerMeta'
import { buildGeometry, fromGeometry, toGeometryType } from './geometry'

const Feature = FlatGeobuf.Feature

export function buildFeature(feature: any, layers: LayerMeta[]) {
    const layerIndex = layers.findIndex(l => l.geometryType === toGeometryType(feature.geometry.type))
    if (layerIndex === -1)
        throw new Error('Cannot introspect to an existing layer')

    const builder = new flatbuffers.Builder(0)
    const geometryOffset = buildGeometry(builder, feature.geometry)
    Feature.startFeature(builder)
    Feature.addLayer(builder, layerIndex)
    Feature.addGeometry(builder, geometryOffset)
    const offset = Feature.endFeature(builder)
    builder.finish(offset)
    return builder.dataBuffer()
}

export function fromFeature(feature: FlatGeobuf.Feature, layers: LayerMeta[]) {
    const layer = layers[feature.layer()]
    const geometry = fromGeometry(feature.geometry(), layer.geometryType)
    return {
        type: 'Feature',
        geometry,
    }
}

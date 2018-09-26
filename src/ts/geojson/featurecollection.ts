import { flatbuffers } from 'flatbuffers'

import ColumnMeta from '../ColumnMeta'
import { FlatGeobuf } from '../flatgeobuf_generated'
import LayerMeta from '../LayerMeta'

import { getInt32, toInt32, toUint8Array } from '../utils'
import { buildFeature, fromFeature } from './feature'
import { toGeometryType } from './geometry'

const Header = FlatGeobuf.Header

const SIZE_PREFIX_LEN: number = 8

export function toFlatGeobuf(featurecollection: any) {

    const layers = introspectLayers(featurecollection)

    const header = toUint8Array(buildHeader(featurecollection, layers))

    const features: Uint8Array[] = featurecollection.features
        .map((f: any) => buildFeature(f, layers))
        .map(toUint8Array)

    const featuresLength = features
        .map(f => SIZE_PREFIX_LEN + f.length)
        .reduce((a, b) => a + b)

    const uint8 = new Uint8Array(SIZE_PREFIX_LEN + header.length + featuresLength)
    uint8.set(toInt32(header.length), 0)
    uint8.set(header, SIZE_PREFIX_LEN)
    let offset = SIZE_PREFIX_LEN + header.length
    for (const feature of features) {
        uint8.set(toInt32(feature.length), offset)
        uint8.set(feature, offset + SIZE_PREFIX_LEN)
        offset += SIZE_PREFIX_LEN + feature.length
    }
    return uint8
}

export function fromFlatGeobuf(bytes: Uint8Array) {
    const headerLength = getInt32(bytes, 0)

    const headerBytes = new Uint8Array(bytes.buffer, SIZE_PREFIX_LEN)
    let offset = SIZE_PREFIX_LEN + headerLength

    const bb = new flatbuffers.ByteBuffer(headerBytes)
    const header = FlatGeobuf.Header.getRootAsHeader(bb)
    const count = header.featuresCount().toFloat64()

    const layers: LayerMeta[] = []
    for (let i = 0; i < header.layersLength(); i++) {
        const layer = header.layers(i)
        layers.push(new LayerMeta(layer.geometryType(), null))
    }

    const features = []
    for (let i = 0; i < count; i++) {
        const featureDataBytes = new Uint8Array(bytes.buffer, offset)
        const featureLength = getInt32(featureDataBytes, offset)
        const featureBytes = new Uint8Array(bytes.buffer, offset + SIZE_PREFIX_LEN)
        const featureBB = new flatbuffers.ByteBuffer(featureBytes)
        const feature = FlatGeobuf.Feature.getRootAsFeature(featureBB)
        features.push(fromFeature(feature, layers))
        offset += (SIZE_PREFIX_LEN + featureLength)
    }

    return {
        type: 'FeatureCollection',
        features,
    }
}

function buildLayer(builder: flatbuffers.Builder, layer: LayerMeta) {
    FlatGeobuf.Layer.startLayer(builder)
    // if (columnsOffset.HasValue)
    //    Layer.AddColumns(builder, columnsOffset.Value);
    FlatGeobuf.Layer.addGeometryType(builder, layer.geometryType)
    const layerOffset = FlatGeobuf.Layer.endLayer(builder)
    return layerOffset
}

function buildHeader(featurecollection: any, layers: LayerMeta[]) {
    const length = featurecollection.features.length
    const builder = new flatbuffers.Builder(0)

    const layerOffsets = layers.map(l => buildLayer(builder, l))
    const layersOffset = Header.createLayersVector(builder, layerOffsets)

    Header.startHeader(builder)
    Header.addFeaturesCount(builder, new flatbuffers.Long(length, 0))
    Header.addLayers(builder, layersOffset)
    const offset = Header.endHeader(builder)
    builder.finish(offset)
    return builder.dataBuffer()
}

function introspectLayers(featurecollection: any) {
    // const feature = featurecollection.features[0]
    const columns: ColumnMeta[] = null
    /*
    if (feature.properties != null && feature.properties.length > 0) {
        Object.keys(feature.properties)
            .map(k => { })
        columns = featureFirst.Attributes.GetNames()
            .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
            .ToList();
    }
    */

    const geometryTypeNamesSet = new Set()
    for (const f of featurecollection.features) {
        geometryTypeNamesSet.add(f.geometry.type)
    }

    const layers = Array.from(geometryTypeNamesSet)
        .map(n => new LayerMeta(toGeometryType(n), columns))

    return layers
}

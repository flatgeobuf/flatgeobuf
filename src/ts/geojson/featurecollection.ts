import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

import { getInt32, toInt32, toUint8Array } from '../utils'
import { buildFeature, fromFeature } from './feature'

const Header = FlatGeobuf.Header

export function toFlatGeobuf(featurecollection: any) {
    const header = toUint8Array(buildHeader(featurecollection))

    const features: Uint8Array[] = featurecollection.features
        .map(buildFeature)
        .map(toUint8Array)

    const featuresLength = features
        .map(a => a.length + 4)
        .reduce((a, b) => a + b)

    const uint8 = new Uint8Array(4 + header.length + featuresLength)
    uint8.set(toInt32(header.length), 0)
    uint8.set(header, 4)
    let offset = 4 + header.length
    for (const feature of features) {
        uint8.set(toInt32(feature.length), offset)
        uint8.set(feature, offset + 4)
        offset += 4 + feature.length
    }
    return uint8
}

export function fromFlatGeobuf(bytes: Uint8Array) {
    const headerLength = getInt32(bytes)

    let offset = 4
    const headerBytes = new Uint8Array(bytes.buffer, offset)
    offset += headerLength

    const bb = new flatbuffers.ByteBuffer(headerBytes)
    const header = FlatGeobuf.Header.getRootAsHeader(bb)
    const count = header.featuresCount().toFloat64()

    const features = []
    for (let i = 0; i < count; i++) {
        const featureDataBytes = new Uint8Array(bytes.buffer, offset)
        const featureLength = getInt32(featureDataBytes)
        offset += 4
        const featureBytes = new Uint8Array(featureDataBytes.buffer, offset)
        const featureBB = new flatbuffers.ByteBuffer(featureBytes)
        const feature = FlatGeobuf.Feature.getRootAsFeature(featureBB)
        features.push(fromFeature(feature))
        offset += featureLength
    }

    return {
        type: 'FeatureCollection',
        features,
    }
}

function buildHeader(featurecollection: any) {
    const length = featurecollection.features.length
    const builder = new flatbuffers.Builder(0)
    Header.startHeader(builder)
    Header.addFeaturesCount(builder, new flatbuffers.Long(length, 0))
    const offset = Header.endHeader(builder)
    builder.finish(offset)
    return builder.dataBuffer()
}

import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

import { getInt32, toInt32, toUint8Array } from '../utils'
import { buildFeature, fromFeature } from './feature'

const Header = FlatGeobuf.Header

const LEN: number = 8

export function toFlatGeobuf(featurecollection: any) {
    const header = toUint8Array(buildHeader(featurecollection))

    const features: Uint8Array[] = featurecollection.features
        .map(buildFeature)
        .map(toUint8Array)

    const featuresLength = features
        .map(f => LEN + f.length)
        .reduce((a, b) => a + b)

    const uint8 = new Uint8Array(LEN + header.length + featuresLength)
    uint8.set(toInt32(header.length), 0)
    uint8.set(header, LEN)
    let offset = LEN + header.length
    for (const feature of features) {
        uint8.set(toInt32(feature.length), offset)
        uint8.set(feature, offset + LEN)
        offset += LEN + feature.length
    }
    return uint8
}

export function fromFlatGeobuf(bytes: Uint8Array) {
    const headerLength = getInt32(bytes, 0)

    const headerBytes = new Uint8Array(bytes.buffer, LEN)
    let offset = LEN + headerLength

    const bb = new flatbuffers.ByteBuffer(headerBytes)
    const header = FlatGeobuf.Header.getRootAsHeader(bb)
    const count = header.featuresCount().toFloat64()

    const features = []
    for (let i = 0; i < count; i++) {
        const featureDataBytes = new Uint8Array(bytes.buffer, offset)
        const featureLength = getInt32(featureDataBytes, offset)
        const featureBytes = new Uint8Array(bytes.buffer, offset + LEN)
        const featureBB = new flatbuffers.ByteBuffer(featureBytes)
        const feature = FlatGeobuf.Feature.getRootAsFeature(featureBB)
        features.push(fromFeature(feature))
        offset += (LEN + featureLength)
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

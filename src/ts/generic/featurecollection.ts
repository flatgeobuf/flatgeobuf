import { flatbuffers } from 'flatbuffers'
import { ReadableStream } from 'web-streams-polyfill/ponyfill'
import slice from 'slice-source/index.js'

import ColumnMeta from '../ColumnMeta'
import ColumnType from '../ColumnType'
import { Header, Column } from '../header_generated'
import { Feature } from '../feature_generated'
import HeaderMeta from '../HeaderMeta'

import { buildFeature, fromFeature, ICreateFeature, IFeature } from './feature'
import { toGeometryType, ICreateGeometry } from './geometry'
import * as tree from '../packedrtree'

const SIZE_PREFIX_LEN: number = 4
const FEATURE_OFFSET_LEN: number = 8

const magicbytes: Uint8Array = new Uint8Array([0x66, 0x67, 0x62, 0x00, 0x66, 0x67, 0x62, 0x00]);

export function serialize(features: IFeature[]) {
    const headerMeta = introspectHeaderMeta(features)
    const header = buildHeader(features, headerMeta)
    const featureBuffers: Uint8Array[] = features
        .map(f => buildFeature(f, headerMeta))
    const featuresLength = featureBuffers
        .map(f => f.length)
        .reduce((a, b) => a + b)
    const uint8 = new Uint8Array(magicbytes.length + header.length + featuresLength)
    uint8.set(header, magicbytes.length)
    let offset = magicbytes.length + header.length
    for (const feature of featureBuffers) {
        uint8.set(feature, offset)
        offset += feature.length
    }
    uint8.set(magicbytes)
    return uint8
}

export function deserialize(
        bytes: Uint8Array,
        createGeometry: ICreateGeometry,
        createFeature: ICreateFeature): IFeature[] {
    if (!bytes.subarray(0, 7).every((v, i) => magicbytes[i] === v))
        throw new Error('Not a FlatGeobuf file')

    const bb = new flatbuffers.ByteBuffer(bytes)
    const headerLength = bb.readUint32(magicbytes.length)
    bb.setPosition(magicbytes.length + SIZE_PREFIX_LEN)
    const header = Header.getRoot(bb)
    const count = header.featuresCount().toFloat64()

    const columns: ColumnMeta[] = []
    for (let j = 0; j < header.columnsLength(); j++) {
        const column = header.columns(j)
        columns.push(new ColumnMeta(column.name(), column.type()))
    }
    const headerMeta = new HeaderMeta(header.geometryType(), columns)

    let offset = magicbytes.length + SIZE_PREFIX_LEN + headerLength

    const indexNodeSize = header.indexNodeSize()
    if (indexNodeSize > 0)
        offset += tree.size(count, indexNodeSize) + (count * FEATURE_OFFSET_LEN)

    const features = []
    for (let i = 0; i < count; i++) {
        const bb = new flatbuffers.ByteBuffer(bytes)
        const featureLength = bb.readUint32(offset)
        bb.setPosition(offset + SIZE_PREFIX_LEN)
        const feature = Feature.getRoot(bb)
        features.push(fromFeature(feature, headerMeta, createGeometry, createFeature))
        offset += SIZE_PREFIX_LEN + featureLength
    }

    return features
}

export async function* deserializeStream(
        stream: ReadableStream,
        createGeometry: ICreateGeometry,
        createFeature: ICreateFeature) {
    const reader = slice(stream)
    let bytes = await reader.slice(8)
    if (!bytes.every((v, i) => magicbytes[i] === v))
        throw new Error('Not a FlatGeobuf file')
    bytes = await reader.slice(4)
    let bb = new flatbuffers.ByteBuffer(bytes)
    const headerLength = bb.readUint32(0)
    bytes = await reader.slice(headerLength)
    bb = new flatbuffers.ByteBuffer(bytes)
    const header = Header.getRoot(bb)
    const count = header.featuresCount().toFloat64()

    const columns: ColumnMeta[] = []
    for (let j = 0; j < header.columnsLength(); j++) {
        const column = header.columns(j)
        columns.push(new ColumnMeta(column.name(), column.type()))
    }
    const headerMeta = new HeaderMeta(header.geometryType(), columns)

    const indexNodeSize = header.indexNodeSize()
    if (indexNodeSize > 0)
        await reader.slice(tree.size(count, indexNodeSize) + (count * FEATURE_OFFSET_LEN))

    for (let i = 0; i < count; i++) {
        bytes = await reader.slice(4)
        bb = new flatbuffers.ByteBuffer(bytes)
        const featureLength = bb.readUint32(0)
        bytes = await reader.slice(featureLength)
        const bytesAligned = new Uint8Array(featureLength + 4)
        bytesAligned.set(bytes, 4)
        bb = new flatbuffers.ByteBuffer(bytesAligned)
        bb.setPosition(SIZE_PREFIX_LEN)
        const feature = Feature.getRoot(bb)
        yield fromFeature(feature, headerMeta, createGeometry, createFeature)
    }
}

function buildColumn(builder: flatbuffers.Builder, column: ColumnMeta) {
    const nameOffset = builder.createString(column.name)
    Column.start(builder)
    Column.addName(builder, nameOffset)
    Column.addType(builder, column.type)
    return Column.end(builder)
}

function buildHeader(features: any, header: HeaderMeta) {
    const length = features.length
    const builder = new flatbuffers.Builder(0)

    let columnOffsets = null
    if (header.columns)
        columnOffsets = Header.createColumnsVector(builder,
            header.columns.map(c => buildColumn(builder, c)))

    const nameOffset = builder.createString('L1')

    Header.start(builder)
    Header.addFeaturesCount(builder, new flatbuffers.Long(length, 0))
    Header.addGeometryType(builder, header.geometryType)
    Header.addIndexNodeSize(builder, 0)
    if (columnOffsets)
        Header.addColumns(builder, columnOffsets)
    Header.addName(builder, nameOffset)
    const offset = Header.end(builder)
    builder.finishSizePrefixed(offset)
    return builder.asUint8Array()
}

function valueToType(value: boolean | number | string | object): ColumnType {
    if (typeof value === 'boolean')
        return ColumnType.Bool
    else if (typeof value === 'number')
        if (value % 1 === 0)
            return ColumnType.Int
        else
            return ColumnType.Double
    else if (typeof value === 'string')
        return ColumnType.String
    else if (value === null)
        return ColumnType.String
    else
        throw new Error(`Unknown type (value '${value}')`)
}

function introspectHeaderMeta(features: any) {
    const feature = features[0]
    const geometry = feature.getGeometry()
    const geometryType = geometry.getType()
    const properties = feature.getProperties()

    let columns: ColumnMeta[] = null
    if (properties)
        columns = Object.keys(properties).filter(key => key !== 'geometry')
            .map(k => new ColumnMeta(k, valueToType(properties[k])))

    const geometryTypeNamesSet = new Set()
    for (const f of features)
        geometryTypeNamesSet.add(geometryType)

    const headerMeta = new HeaderMeta(toGeometryType(geometryType), columns)

    return headerMeta
}

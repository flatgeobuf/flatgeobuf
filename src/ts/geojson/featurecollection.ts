import { flatbuffers } from 'flatbuffers'

import ColumnMeta from '../ColumnMeta'
import ColumnType from '../ColumnType'
import { FlatGeobuf } from '../flatgeobuf_generated'
import HeaderMeta from '../HeaderMeta'

import { getInt32, toInt32, toUint8Array } from '../utils'
import { buildFeature, fromFeature, IGeoJsonFeature } from './feature'
import { toGeometryType } from './geometry'
import * as tree from '../packedrtree'

const Header = FlatGeobuf.Header
const Column = FlatGeobuf.Column

const SIZE_PREFIX_LEN: number = 4
const FEATURE_OFFSET_LEN: number = 8

const magicbytes: Uint8Array  = new Uint8Array([0x66, 0x67, 0x62, 0x00]);

export interface IGeoJsonFeatureCollection {
    type: string,
    features?: IGeoJsonFeature[]
}

export function serialize(featurecollection: IGeoJsonFeatureCollection) {
    const headerMeta = introspectHeaderMeta(featurecollection)
    const header = toUint8Array(buildHeader(featurecollection, headerMeta))

    const features: Uint8Array[] = featurecollection.features
        .map(f => buildFeature(f, headerMeta))
        .map(toUint8Array)

    const featuresLength = features
        .map(f => SIZE_PREFIX_LEN + f.length)
        .reduce((a, b) => a + b)

    const uint8 = new Uint8Array(4 + SIZE_PREFIX_LEN + header.length + featuresLength)
    uint8.set(toInt32(header.length), 4)
    uint8.set(header, 4 + SIZE_PREFIX_LEN)
    let offset = 4 + SIZE_PREFIX_LEN + header.length
    for (const feature of features) {
        uint8.set(toInt32(feature.length), offset)
        uint8.set(feature, offset + SIZE_PREFIX_LEN)
        offset += SIZE_PREFIX_LEN + feature.length
    }
    uint8.set(magicbytes)
    return uint8
}

export function deserialize(bytes: Uint8Array) {
    if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v))
        throw new Error('Not a FlatGeobuf file')

    const headerLength = getInt32(bytes, 4)

    const headerBytes = new Uint8Array(bytes.buffer, 4 + SIZE_PREFIX_LEN)
    let offset = 4 + SIZE_PREFIX_LEN + headerLength

    const bb = new flatbuffers.ByteBuffer(headerBytes)
    const header = FlatGeobuf.Header.getRootAsHeader(bb)
    const count = header.featuresCount().toFloat64()

    const columns: ColumnMeta[] = []
    for (let j = 0; j < header.columnsLength(); j++) {
        const column = header.columns(j)
        columns.push(new ColumnMeta(column.name(), column.type()))
    }
    const headerMeta = new HeaderMeta(header.geometryType(), columns)

    const index = header.index()
    if (index) {
        const nodeSize = index.nodeSize()
        offset += tree.size(count, index.nodeSize()) + (count * FEATURE_OFFSET_LEN)
    }

    const features = []
    for (let i = 0; i < count; i++) {
        const featureDataBytes = new Uint8Array(bytes.buffer, offset)
        const featureLength = getInt32(featureDataBytes, offset)
        const featureBytes = new Uint8Array(bytes.buffer.slice(offset + SIZE_PREFIX_LEN))
        const featureBB = new flatbuffers.ByteBuffer(featureBytes)
        const feature = FlatGeobuf.Feature.getRootAsFeature(featureBB)
        features.push(fromFeature(feature, headerMeta))
        offset += (SIZE_PREFIX_LEN + featureLength)
    }

    return {
        type: 'FeatureCollection',
        features,
    } as IGeoJsonFeatureCollection
}

function buildColumn(builder: flatbuffers.Builder, column: ColumnMeta) {
    const nameOffset = builder.createString(column.name)
    Column.startColumn(builder)
    Column.addName(builder, nameOffset)
    Column.addType(builder, column.type)
    return Column.endColumn(builder)
}

function buildHeader(featurecollection: IGeoJsonFeatureCollection, header: HeaderMeta) {
    const length = featurecollection.features.length
    const builder = new flatbuffers.Builder(0)

    let columnOffsets = null
    if (header.columns)
        columnOffsets = Header.createColumnsVector(builder,
            header.columns.map(c => buildColumn(builder, c)))

    const nameOffset = builder.createString('L1')

    Header.startHeader(builder)
    Header.addFeaturesCount(builder, new flatbuffers.Long(length, 0))
    Header.addGeometryType(builder, header.geometryType)
    if (columnOffsets)
        Header.addColumns(builder, columnOffsets)
    Header.addName(builder, nameOffset)
    const offset = Header.endHeader(builder)
    builder.finish(offset)
    return builder.dataBuffer()
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

function introspectHeaderMeta(featurecollection: IGeoJsonFeatureCollection) {
    const feature = featurecollection.features[0]
    const properties = feature.properties

    let columns: ColumnMeta[] = null
    if (properties)
        columns = Object.keys(properties)
            .map(k => new ColumnMeta(k, valueToType(properties[k])))

    const geometryTypeNamesSet = new Set()
    for (const f of featurecollection.features)
        geometryTypeNamesSet.add(feature.geometry.type)

    const headerMeta = new HeaderMeta(toGeometryType(feature.geometry.type), columns)

    return headerMeta
}

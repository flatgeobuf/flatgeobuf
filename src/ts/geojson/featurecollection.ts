import { flatbuffers } from 'flatbuffers'

import ColumnMeta from '../ColumnMeta'
import ColumnType from '../ColumnType'
import { Header, Column } from '../header_generated'
import { Feature } from '../feature_generated'
import HeaderMeta from '../HeaderMeta'

import { buildFeature, fromFeature, IGeoJsonFeature } from './feature'
import { toGeometryType } from './geometry'
import * as tree from '../packedrtree'

const SIZE_PREFIX_LEN: number = 4
const FEATURE_OFFSET_LEN: number = 8

const magicbytes: Uint8Array  = new Uint8Array([0x66, 0x67, 0x62, 0x00, 0x66, 0x67, 0x62, 0x00]);

export interface IGeoJsonFeatureCollection {
    type: string,
    features?: IGeoJsonFeature[]
}

export function serialize(featurecollection: IGeoJsonFeatureCollection) {
    const headerMeta = introspectHeaderMeta(featurecollection)
    const header = buildHeader(featurecollection, headerMeta)
    const features: Uint8Array[] = featurecollection.features
        .map(f => buildFeature(f, headerMeta))
    const featuresLength = features
        .map(f => f.length)
        .reduce((a, b) => a + b)
    const uint8 = new Uint8Array(magicbytes.length + header.length + featuresLength)
    uint8.set(header, magicbytes.length)
    let offset = magicbytes.length + header.length
    for (const feature of features) {
        uint8.set(feature, offset)
        offset += feature.length
    }
    uint8.set(magicbytes)
    return uint8
}

export function deserialize(bytes: Uint8Array) {
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
        features.push(fromFeature(feature, headerMeta))
        offset += SIZE_PREFIX_LEN + featureLength
    }

    return {
        type: 'FeatureCollection',
        features,
    } as IGeoJsonFeatureCollection
}

function buildColumn(builder: flatbuffers.Builder, column: ColumnMeta) {
    const nameOffset = builder.createString(column.name)
    Column.start(builder)
    Column.addName(builder, nameOffset)
    Column.addType(builder, column.type)
    return Column.end(builder)
}

function buildHeader(featurecollection: IGeoJsonFeatureCollection, header: HeaderMeta) {
    const length = featurecollection.features.length
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

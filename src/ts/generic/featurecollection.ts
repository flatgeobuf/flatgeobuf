import * as flatbuffers from 'flatbuffers'
import slice from 'slice-source'

import ColumnMeta from '../ColumnMeta'

import { Header } from '../header'

import { Column } from '../column'
import { ColumnType } from '../column-type'
import { Feature } from '../feature'
import HeaderMeta from '../HeaderMeta'

import { buildFeature, IFeature } from './feature'
import { toGeometryType } from './geometry'
import { HttpReader} from '../HttpReader';
import Logger from '../Logger'
import { Rect, calcTreeSize} from '../packedrtree'
import { parseGeometry } from './geometry'
import { HeaderMetaFn} from '../generic'
import { magicbytes, SIZE_PREFIX_LEN } from '../constants'

export type FromFeatureFn = (feature: Feature, header: HeaderMeta) => IFeature
type ReadFn = (size: number, purpose: string) => Promise<ArrayBuffer>

export function serialize(features: IFeature[]) : Uint8Array {
    const headerMeta = introspectHeaderMeta(features)
    const header = buildHeader(headerMeta)
    const featureBuffers: Uint8Array[] = features
        .map(f => {
            if (!f.getGeometry)
                throw new Error('Missing getGeometry implementation')
            if (!f.getProperties)
                throw new Error('Missing getProperties implementation')
            return buildFeature(parseGeometry(f.getGeometry(), headerMeta.geometryType), f.getProperties(), headerMeta)
        })
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

export function deserialize(bytes: Uint8Array, fromFeature: FromFeatureFn, headerMetaFn?: HeaderMetaFn) : IFeature[] {
    if (!bytes.subarray(0, 7).every((v, i) => magicbytes[i] === v))
        throw new Error('Not a FlatGeobuf file')

    const bb = new flatbuffers.ByteBuffer(bytes)
    const headerLength = bb.readUint32(magicbytes.length)
    bb.setPosition(magicbytes.length + SIZE_PREFIX_LEN)

    const headerMeta = HeaderMeta.fromByteBuffer(bb);
    if (headerMetaFn)
        headerMetaFn(headerMeta)

    let offset = magicbytes.length + SIZE_PREFIX_LEN + headerLength

    const { indexNodeSize, featuresCount } = headerMeta
    if (indexNodeSize > 0)
        offset += calcTreeSize(featuresCount, indexNodeSize)

    const features: IFeature[] = []
    while (offset < bb.capacity()) {
        const featureLength = bb.readUint32(offset)
        bb.setPosition(offset + SIZE_PREFIX_LEN)
        const feature = Feature.getRootAsFeature(bb)
        features.push(fromFeature(feature, headerMeta))
        offset += SIZE_PREFIX_LEN + featureLength
    }

    return features
}

export async function* deserializeStream(stream: ReadableStream, fromFeature: FromFeatureFn, headerMetaFn?: HeaderMetaFn)
    : AsyncGenerator<IFeature>
{
    const reader = slice(stream)
    const read: ReadFn = async size => await reader.slice(size)

    let bytes = new Uint8Array(await read(8, 'magic bytes'))
    if (!bytes.every((v, i) => magicbytes[i] === v))
        throw new Error('Not a FlatGeobuf file')
    bytes = new Uint8Array(await read(4, 'header length'))
    let bb = new flatbuffers.ByteBuffer(bytes)
    const headerLength = bb.readUint32(0)
    bytes = new Uint8Array(await read(headerLength, 'header data'))
    bb = new flatbuffers.ByteBuffer(bytes)

    const headerMeta = HeaderMeta.fromByteBuffer(bb);
    if (headerMetaFn)
        headerMetaFn(headerMeta)

    const { indexNodeSize, featuresCount } = headerMeta
    if (indexNodeSize > 0) {
        const treeSize = calcTreeSize(featuresCount, indexNodeSize)
        await read(treeSize, 'entire index, w/o rect')
    }
    let feature : IFeature | undefined
    while ((feature = await readFeature(read, headerMeta, fromFeature)))
        yield feature
}

export async function *deserializeFiltered(url: string, rect: Rect, fromFeature: FromFeatureFn, headerMetaFn?: HeaderMetaFn)
    : AsyncGenerator<IFeature>
{
        const reader = await HttpReader.open(url)
        Logger.debug('opened reader')
        if (headerMetaFn)
            headerMetaFn(reader.header)

        for await (const featureOffset of reader.selectBbox(rect)) {
            const feature = await reader.readFeature(featureOffset[0])
            yield fromFeature(feature, reader.header)
        }
}

async function readFeature(read: ReadFn, headerMeta: HeaderMeta, fromFeature: FromFeatureFn): Promise<IFeature | undefined> {
    let bytes = new Uint8Array(await read(4, 'feature length'))
    if (bytes.byteLength === 0)
        return
    let bb = new flatbuffers.ByteBuffer(bytes)
    const featureLength = bb.readUint32(0)
    bytes = new Uint8Array(await read(featureLength, 'feature data'))
    const bytesAligned = new Uint8Array(featureLength + 4)
    bytesAligned.set(bytes, 4)
    bb = new flatbuffers.ByteBuffer(bytesAligned)
    bb.setPosition(SIZE_PREFIX_LEN)
    const feature = Feature.getRootAsFeature(bb)
    return fromFeature(feature, headerMeta)
}

function buildColumn(builder: flatbuffers.Builder, column: ColumnMeta): number {
    const nameOffset = builder.createString(column.name)
    Column.startColumn(builder)
    Column.addName(builder, nameOffset)
    Column.addType(builder, column.type)
    return Column.endColumn(builder)
}

export function buildHeader(header: HeaderMeta): Uint8Array {
    const builder = new flatbuffers.Builder()

    let columnOffsets = null
    if (header.columns)
        columnOffsets = Header.createColumnsVector(builder,
            header.columns.map(c => buildColumn(builder, c)))

    const nameOffset = builder.createString('L1')

    Header.startHeader(builder)
    Header.addFeaturesCount(builder, new flatbuffers.Long(header.featuresCount, 0))
    Header.addGeometryType(builder, header.geometryType)
    Header.addIndexNodeSize(builder, 0)
    if (columnOffsets)
        Header.addColumns(builder, columnOffsets)
    Header.addName(builder, nameOffset)
    const offset = Header.endHeader(builder)
    builder.finishSizePrefixed(offset)
    return builder.asUint8Array() as Uint8Array
}

function valueToType(value: boolean | number | string): ColumnType {
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

function introspectHeaderMeta(features: IFeature[]): HeaderMeta {
    const feature = features[0]
    const geometry = feature.getGeometry ? feature.getGeometry() : undefined
    const geometryType = geometry ? geometry.getType() : undefined
    const properties = feature.getProperties ? feature.getProperties() : {}

    let columns: ColumnMeta[] | null = null
    if (properties)
        columns = Object.keys(properties).filter(key => key !== 'geometry')
            .map(k => new ColumnMeta(k, valueToType(properties[k]), null, null, -1, -1, -1, true, false, false))

    const headerMeta = new HeaderMeta(toGeometryType(geometryType), columns, features.length, 0, null, null, null, null)
    return headerMeta
}

import * as flatbuffers from 'flatbuffers';
import slice from 'slice-source';
import { ArrayReader } from '../array-reader.js';
import type { ColumnMeta } from '../column-meta.js';
import { magicbytes, SIZE_PREFIX_LEN } from '../constants.js';
import { Column } from '../flat-geobuf/column.js';
import { ColumnType } from '../flat-geobuf/column-type.js';
import { Crs } from '../flat-geobuf/crs.js';
import { Feature } from '../flat-geobuf/feature.js';
import { Header } from '../flat-geobuf/header.js';
import type { HeaderMetaFn } from '../generic.js';
import type { HeaderMeta } from '../header-meta.js';
import { fromByteBuffer } from '../header-meta.js';
import { HttpRangeClient, HttpReader } from '../http-reader.js';
import { calcTreeSize, type Rect } from '../packedrtree.js';
import { buildFeature, type IFeature, type IProperties } from './feature.js';
import { parseGeometry } from './geometry.js';
import { inferGeometryType } from './header.js';

export type FromFeatureFn = (id: number, feature: Feature, header: HeaderMeta) => IFeature;
type ReadFn = (size: number, purpose: string) => Promise<ArrayBuffer | Uint8Array>;

/**
 * Serialize generic features to FlatGeobuf
 * @param features
 */
export function serialize(features: IFeature[]): Uint8Array {
    const headerMeta = introspectHeaderMeta(features);
    const header = buildHeader(headerMeta);
    const featureBuffers: Uint8Array[] = features.map((f) => {
        if (!f.getGeometry) throw new Error('Missing getGeometry implementation');
        if (!f.getProperties) throw new Error('Missing getProperties implementation');
        return buildFeature(parseGeometry(f.getGeometry(), headerMeta.geometryType), f.getProperties(), headerMeta);
    });
    const featuresLength = featureBuffers.map((f) => f.length).reduce((a, b) => a + b);
    const uint8 = new Uint8Array(magicbytes.length + header.length + featuresLength);
    uint8.set(header, magicbytes.length);
    let offset = magicbytes.length + header.length;
    for (const feature of featureBuffers) {
        uint8.set(feature, offset);
        offset += feature.length;
    }
    uint8.set(magicbytes);
    return uint8;
}

export async function* deserialize(
    bytes: Uint8Array,
    fromFeature: FromFeatureFn,
    rect?: Rect,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IFeature> {
    if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v)) throw new Error('Not a FlatGeobuf file');

    if (rect) {
        const reader = ArrayReader.open(bytes);
        for await (const feature of reader.selectBbox(rect)) {
            yield fromFeature(feature.id, feature.feature, reader.header);
        }
        return;
    }

    const bb = new flatbuffers.ByteBuffer(bytes);
    const headerLength = bb.readUint32(magicbytes.length);
    bb.setPosition(magicbytes.length);

    const headerMeta = fromByteBuffer(bb);
    if (headerMetaFn) headerMetaFn(headerMeta);

    let offset = magicbytes.length + SIZE_PREFIX_LEN + headerLength;

    const { indexNodeSize, featuresCount } = headerMeta;
    if (indexNodeSize > 0) offset += calcTreeSize(featuresCount, indexNodeSize);

    let id = 0;
    while (offset < bb.capacity()) {
        const featureLength = bb.readUint32(offset);
        bb.setPosition(offset);
        const feature = Feature.getSizePrefixedRootAsFeature(bb);
        yield fromFeature(id++, feature, headerMeta);
        offset += SIZE_PREFIX_LEN + featureLength;
    }
}

export async function* deserializeStream(
    stream: ReadableStream,
    fromFeature: FromFeatureFn,
    headerMetaFn?: HeaderMetaFn,
): AsyncGenerator<IFeature> {
    const reader = slice(stream);
    const read: ReadFn = async (size) => await reader.slice(size);

    let bytes = new Uint8Array(await read(8, 'magic bytes'));
    if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v)) throw new Error('Not a FlatGeobuf file');
    const headerLengthBytes = new Uint8Array(await read(4, 'header length'));
    let bb = new flatbuffers.ByteBuffer(headerLengthBytes);
    const headerLength = bb.readUint32(0);
    const headerDataBytes = new Uint8Array(await read(headerLength, 'header data'));
    bytes = new Uint8Array(headerLengthBytes.length + headerDataBytes.length);
    bytes.set(headerLengthBytes);
    bytes.set(headerDataBytes, headerLengthBytes.length);
    bb = new flatbuffers.ByteBuffer(bytes);

    const headerMeta = fromByteBuffer(bb);
    if (headerMetaFn) headerMetaFn(headerMeta);

    const { indexNodeSize, featuresCount } = headerMeta;
    if (indexNodeSize > 0) {
        const treeSize = calcTreeSize(featuresCount, indexNodeSize);
        await read(treeSize, 'entire index, w/o rect');
    }
    let feature: IFeature | undefined;
    let id = 0;
    while ((feature = await readFeature(read, headerMeta, fromFeature, id++))) yield feature;
}

export async function* deserializeFiltered(
    url: string,
    rect: Rect,
    fromFeature: FromFeatureFn,
    headerMetaFn?: HeaderMetaFn,
    nocache = false,
    headers: HeadersInit = {},
): AsyncGenerator<IFeature> {
    const reader = await HttpReader.open(url, nocache, headers);
    console.debug('opened reader');
    if (headerMetaFn) headerMetaFn(reader.header);
    for await (const feature of reader.selectBbox(rect)) yield fromFeature(feature.id, feature.feature, reader.header);
}

async function readFeature(
    read: ReadFn,
    headerMeta: HeaderMeta,
    fromFeature: FromFeatureFn,
    id: number,
): Promise<IFeature | undefined> {
    let bytes = new Uint8Array(await read(4, 'feature length'));
    if (bytes.byteLength === 0) return;
    let bb = new flatbuffers.ByteBuffer(bytes);
    const featureLength = bb.readUint32(0);
    bytes = new Uint8Array(await read(featureLength, 'feature data'));
    const bytesAligned = new Uint8Array(featureLength + 4);
    bytesAligned.set(bytes, 4);
    bb = new flatbuffers.ByteBuffer(bytesAligned);
    const feature = Feature.getSizePrefixedRootAsFeature(bb);
    return fromFeature(id, feature, headerMeta);
}

function buildColumn(builder: flatbuffers.Builder, column: ColumnMeta): number {
    const nameOffset = builder.createString(column.name);
    Column.startColumn(builder);
    Column.addName(builder, nameOffset);
    Column.addType(builder, column.type);
    return Column.endColumn(builder);
}

export function buildHeader(header: HeaderMeta, crsCode = 0): Uint8Array {
    const builder = new flatbuffers.Builder();

    let columnOffsets = 0;
    if (header.columns)
        columnOffsets = Header.createColumnsVector(
            builder,
            header.columns.map((c) => buildColumn(builder, c)),
        );

    const nameOffset = builder.createString('L1');

    let crsOffset: flatbuffers.Offset | undefined;
    if (crsCode) {
        Crs.startCrs(builder);
        Crs.addCode(builder, crsCode);
        crsOffset = Crs.endCrs(builder);
    }
    Header.startHeader(builder);
    if (crsOffset) Header.addCrs(builder, crsOffset);
    Header.addFeaturesCount(builder, BigInt(header.featuresCount));
    Header.addGeometryType(builder, header.geometryType);
    Header.addIndexNodeSize(builder, 0);
    if (columnOffsets) Header.addColumns(builder, columnOffsets);
    Header.addName(builder, nameOffset);
    const offset = Header.endHeader(builder);
    builder.finishSizePrefixed(offset);

    return builder.asUint8Array() as Uint8Array;
}

export async function readMetadata(url: string, nocache = false, headers: HeadersInit = {}): Promise<HeaderMeta> {
    const assumedHeaderLength = 2024;
    const httpClient = new HttpRangeClient(url, nocache, headers);

    const bytes = new Uint8Array(await httpClient.getRange(0, assumedHeaderLength, 'read metadata'));

    if (!bytes.subarray(0, 3).every((v, i) => magicbytes[i] === v)) throw new Error('Not a FlatGeobuf file');

    const bb = new flatbuffers.ByteBuffer(bytes);
    bb.setPosition(magicbytes.length);
    const headerMeta = fromByteBuffer(bb);

    return headerMeta;
}

function valueToType(value: boolean | number | string | Uint8Array | undefined): ColumnType {
    if (typeof value === 'boolean') return ColumnType.Bool;
    if (typeof value === 'number') return ColumnType.Double;
    if (typeof value === 'string') return ColumnType.String;
    if (value === null) return ColumnType.String;
    if (value instanceof Uint8Array) return ColumnType.Binary;
    if (typeof value === 'object') return ColumnType.Json;
    throw new Error(`Unknown type (value '${value}')`);
}

export function mapColumn(properties: IProperties, k: string): ColumnMeta {
    return {
        name: k,
        type: valueToType(properties[k]),
        title: null,
        description: null,
        width: -1,
        precision: -1,
        scale: -1,
        nullable: true,
        unique: false,
        primary_key: false,
    };
}

function introspectHeaderMeta(features: IFeature[]): HeaderMeta {
    const sampleFeature = features[0];
    const properties = sampleFeature.getProperties ? sampleFeature.getProperties() : {};

    let columns: ColumnMeta[] | null = null;
    if (properties)
        columns = Object.keys(properties)
            .filter((key) => key !== 'geometry')
            .map((k) => mapColumn(properties, k));

    const geometryType = inferGeometryType(features);
    const headerMeta: HeaderMeta = {
        geometryType,
        columns,
        envelope: null,
        featuresCount: features.length,
        indexNodeSize: 0,
        crs: null,
        title: null,
        description: null,
        metadata: null,
    };
    return headerMeta;
}

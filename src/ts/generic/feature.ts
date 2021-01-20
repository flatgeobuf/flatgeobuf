import * as flatbuffers from '../flatbuffers/flatbuffers'

import ColumnMeta from '../ColumnMeta'
import { ColumnType } from '../column-type'
import { Feature } from '../feature'
import HeaderMeta from '../HeaderMeta'
import { buildGeometry, ISimpleGeometry, ICreateGeometry, IParsedGeometry } from './geometry'

const textEncoder = new TextEncoder()
const textDecoder = new TextDecoder()

export interface IFeature {
    getGeometry?(): ISimpleGeometry
    getProperties?(): any
    setProperties?(properties: Record<string, unknown>): any
}

export interface ICreateFeature {
    (geometry?: ISimpleGeometry, properties?: Record<string, unknown>): IFeature;
}

export interface IProperties {
    [key: string]: boolean | number | string | any
}

export function fromFeature(
        feature: Feature,
        header: HeaderMeta,
        createGeometry: ICreateGeometry,
        createFeature: ICreateFeature): IFeature {
    const columns = header.columns
    const geometry = feature.geometry()
    const simpleGeometry = createGeometry(geometry, header.geometryType)
    const properties = parseProperties(feature, columns as ColumnMeta[])
    return createFeature(simpleGeometry, properties)
}

export function buildFeature(geometry: IParsedGeometry, properties: IProperties, header: HeaderMeta): Uint8Array {
    const columns = header.columns
    const builder = new flatbuffers.Builder()

    const props = []
    if (columns) {
        for (let i = 0; i < columns.length; i++) {
            const column = columns[i]
            const value = properties[column.name]
            if (value === null)
                continue
            props.push(Uint16Array.of(i))
            switch (column.type) {
                case ColumnType.Bool:
                case ColumnType.Short:
                case ColumnType.UShort:
                case ColumnType.Int:
                case ColumnType.UInt:
                case ColumnType.Long:
                case ColumnType.Double:
                    props.push(column.arrayType.of(value))
                    break
                case ColumnType.DateTime:
                case ColumnType.String: {
                    const str = textEncoder.encode(value)
                    props.push(Uint32Array.of(str.length))
                    props.push(str)
                    break
                }
                default:
                    throw new Error('Unknown type ' + column.type)
            }
        }
    }

    let propertiesOffset = null
    if (props.length > 0)
        propertiesOffset = Feature.createPropertiesVector(builder, concat(Uint8Array, ...props))

    const geometryOffset = buildGeometry(builder, geometry)
    Feature.startFeature(builder)
    Feature.addGeometry(builder, geometryOffset)
    if (propertiesOffset)
        Feature.addProperties(builder, propertiesOffset)
    const featureOffset = Feature.endFeature(builder)
    builder.finishSizePrefixed(featureOffset)
    return builder.asUint8Array() as Uint8Array
}

function concat(resultConstructor: any, ...arrays : any[]) : Uint8Array {
    let totalLength = 0
    for (const arr of arrays)
        totalLength += arr.byteLength
    const result = new resultConstructor(totalLength)
    let offset = 0
    for (const arr of arrays) {
        if (arr instanceof Uint8Array)
            result.set(arr, offset)
        else
            result.set(new resultConstructor(arr.buffer), offset)
        offset += arr.byteLength
    }
    return result
}

export function parseProperties(feature: Feature, columns: ColumnMeta[]): Record<string, unknown> {
    const properties: Record<string, unknown> = {}
    if (!columns || columns.length === 0)
        return properties
    const array = feature.propertiesArray()
    if (!array)
        return properties
    const view = new DataView(array.buffer, array.byteOffset)
    const length = feature.propertiesLength()
    let offset = 0
    while (offset < length) {
        const i = view.getUint16(offset, true)
        offset += 2
        const column = columns[i]
        const name = column.name
        switch (column.type) {
            case ColumnType.Bool: {
                properties[name] = !!view.getUint8(offset)
                offset += 1
                break
            }
            case ColumnType.Byte: {
                properties[name] = view.getInt8(offset)
                offset += 1
                break
            }
            case ColumnType.UByte: {
                properties[name] = view.getUint8(offset)
                offset += 1
                break
            }
            case ColumnType.Short: {
                properties[name] = view.getInt16(offset, true)
                offset += 2
                break
            }
            case ColumnType.UShort: {
                properties[name] = view.getUint16(offset, true)
                offset += 2
                break
            }
            case ColumnType.Int: {
                properties[name] = view.getInt32(offset, true)
                offset += 4
                break
            }
            case ColumnType.UInt: {
                properties[name] = view.getUint32(offset, true)
                offset += 4
                break
            }
            case ColumnType.Long: {
                properties[name] = Number(view.getBigInt64(offset, true))
                offset += 8
                break
            }
            case ColumnType.ULong: {
                properties[name] = Number(view.getBigUint64(offset, true))
                offset += 8
                break
            }
            case ColumnType.Double: {
                properties[name] = view.getFloat64(offset, true)
                offset += 8
                break
            }
            case ColumnType.DateTime:
            case ColumnType.String: {
                const length = view.getUint32(offset, true)
                offset += 4
                properties[name] = textDecoder.decode(array.subarray(offset, offset + length))
                offset += length
                break
            }
            default:
                throw new Error('Unknown type ' + column.type)
        }
    }
    return properties
}

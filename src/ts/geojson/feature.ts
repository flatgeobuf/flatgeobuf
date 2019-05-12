import { flatbuffers } from 'flatbuffers'

import ColumnMeta from '../ColumnMeta'
import ColumnType from '../ColumnType'
import { FlatGeobuf } from '../flatgeobuf_generated'
import HeaderMeta from '../HeaderMeta'
import { buildGeometry, fromGeometry, IGeoJsonGeometry, toGeometryType } from './geometry'

const Feature = FlatGeobuf.Feature
const Value = FlatGeobuf.Value

export interface IGeoJsonProperties {
    [key: string]: boolean | number | string | object
}

export interface IGeoJsonFeature {
    type: string
    geometry: IGeoJsonGeometry
    properties?: IGeoJsonProperties
}

export function buildFeature(feature: IGeoJsonFeature, header: HeaderMeta) {
    const columns = header.columns

    const builder = new flatbuffers.Builder(0)

    let valuesOffset = null
    if (columns) {
        const valueOffsets = columns
            .map((c, i) => buildValue(builder, c, i, feature.properties))
            .filter(v => v !== null)
        valuesOffset = Feature.createValuesVector(builder, valueOffsets)
    }

    const geometryOffset = buildGeometry(builder, feature.geometry)
    Feature.startFeature(builder)
    if (valuesOffset)
        Feature.addValues(builder, valuesOffset)
    Feature.addGeometry(builder, geometryOffset)
    const offset = Feature.endFeature(builder)
    //builder.finishSizePrefixed(offset)
    builder.finish(offset)
    return builder.dataBuffer()
}

function buildValue(
        builder: flatbuffers.Builder,
        column: ColumnMeta,
        columnIndex: number,
        properties: IGeoJsonProperties) {
    const value = properties[column.name]
    if (value === null)
        return
    switch (column.type) {
        case ColumnType.Bool:
            Value.startValue(builder)
            Value.addBoolValue(builder, value as boolean)
            break
        case ColumnType.Int:
            Value.startValue(builder)
            Value.addIntValue(builder, value as number)
            break
        case ColumnType.Double:
            Value.startValue(builder)
            Value.addDoubleValue(builder, value as number)
            break
        case ColumnType.String:
            const stringValue = builder.createString(value as string)
            Value.startValue(builder)
            Value.addStringValue(builder, stringValue)
            break
        default:
            throw new Error('Unknown type')
    }
    Value.addColumnIndex(builder, columnIndex)
    return Value.endValue(builder)
}

export function fromFeature(feature: FlatGeobuf.Feature, header: HeaderMeta) {
    const columns = header.columns
    const geometry = fromGeometry(feature.geometry(), header.geometryType)
    const properties = parseProperties(feature, columns)

    const geoJsonfeature: IGeoJsonFeature = {
        type: 'Feature',
        geometry,
    }
    if (properties)
        geoJsonfeature.properties = properties

    return geoJsonfeature
}

function parseValue(value: FlatGeobuf.Value, column: ColumnMeta) {
    switch (column.type) {
        case ColumnType.Bool: return value.boolValue()
        case ColumnType.Int: return value.intValue()
        case ColumnType.Double: return value.doubleValue()
        case ColumnType.String: return value.stringValue()
    }
}

function parseProperties(feature: FlatGeobuf.Feature, columns: ColumnMeta[]) {
    if (!columns || columns.length === 0)
        return
    const length = feature.valuesLength()
    const properties: IGeoJsonProperties = {}
    for (let i = 0; i < length; i++) {
        const value = feature.values(i)
        const column = columns[value.columnIndex()]
        properties[column.name] = parseValue(value, column)
    }
    return properties
}

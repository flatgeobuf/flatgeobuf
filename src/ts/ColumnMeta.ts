
import { ColumnType } from './header_generated'

const arrayTypeMap = {
    [ColumnType.Byte]: Uint8Array,
    [ColumnType.UByte]: Uint8Array,
    [ColumnType.Bool]: Uint8Array,
    [ColumnType.Short]: Uint16Array,
    [ColumnType.UShort]: Uint16Array,
    [ColumnType.Int]: Uint32Array,
    [ColumnType.UInt]: Uint32Array,
    [ColumnType.Long]: BigUint64Array,
    [ColumnType.ULong]: BigUint64Array,
    [ColumnType.Float]: Float64Array,
    [ColumnType.Double]: Float64Array,
    [ColumnType.String]: String,
    [ColumnType.Json]: String,
    [ColumnType.DateTime]: String,
    [ColumnType.Binary]: String,
}

export default class ColumnMeta {
    name: string
    type: ColumnType
    arrayType: any
    constructor(name: string, type: ColumnType) {
        this.name = name
        this.type = type
        this.arrayType = arrayTypeMap[type] as any
    }
}

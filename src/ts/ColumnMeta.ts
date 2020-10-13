
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
    arrayType: any
    constructor(
        public name: string,
        public type: ColumnType,
        public title: string | null,
        public description: string | null,
        public width: number,
        public precision: number,
        public scale: number,
        public nullable: boolean,
        public unique: boolean,
        public primary_key: boolean
        ) {
        this.arrayType = arrayTypeMap[type]
    }
}

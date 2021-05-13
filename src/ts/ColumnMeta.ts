
import { ColumnType } from './column-type'

export default class ColumnMeta {
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
    }
}

import ColumnMeta from './ColumnMeta'
import CrsMeta from './CrsMeta'
import { GeometryType } from './header_generated'

export default class HeaderMeta {
    constructor(
        public geometryType: GeometryType,
        public columns: ColumnMeta[] | null,
        public featuresCount: number,
        public indexNodeSize: number,
        public crs: CrsMeta | null,
        public title: string | null,
        public description: string | null,
        public metadata: string | null
        ) {
    }
}

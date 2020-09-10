import ColumnMeta from './ColumnMeta'
import CrsMeta from './CrsMeta'
import { GeometryType } from './header_generated'

export default class HeaderMeta {
    constructor(
        public geometryType: GeometryType,
        public columns: ColumnMeta[],
        public featuresCount: number,
        public crs: CrsMeta,
        public title: string,
        public description: string,
        public metadata: string,
        ) {
    }
}

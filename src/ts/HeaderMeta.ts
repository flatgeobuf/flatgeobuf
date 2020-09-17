import ColumnMeta from './ColumnMeta'
import CrsMeta from './CrsMeta'
import { GeometryType } from './header_generated'

export default class HeaderMeta {
    public geometryType: GeometryType
    public columns: ColumnMeta[] | null
    public featuresCount: number
    public crs: CrsMeta | null
    constructor(geometryType: GeometryType, columns: ColumnMeta[] | null, featuresCount: number, crs: CrsMeta | null) {
        this.geometryType = geometryType
        this.columns = columns
        this.featuresCount = featuresCount
        this.crs = crs
    }
}

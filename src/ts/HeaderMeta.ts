import ColumnMeta from './ColumnMeta'
import CrsMeta from './CrsMeta'
import { GeometryType } from './header_generated'

export default class HeaderMeta {
    public geometryType: GeometryType
    public columns: ColumnMeta[]
    public featuresCount: number
    public crs: CrsMeta
    constructor(geometryType: GeometryType, columns: ColumnMeta[], featuresCount: number, crs?: CrsMeta) {
        this.geometryType = geometryType
        this.columns = columns
        this.featuresCount = featuresCount
        this.crs = crs
    }
}

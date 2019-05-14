import ColumnMeta from './ColumnMeta'
import { GeometryType } from './header_generated'

export default class HeaderMeta {
    public geometryType: GeometryType
    public columns: ColumnMeta[]
    constructor(geometryType: GeometryType, columns: ColumnMeta[]) {
        this.geometryType = geometryType
        this.columns = columns
    }
}

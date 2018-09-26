import ColumnMeta from './ColumnMeta'
import { FlatGeobuf } from './flatgeobuf_generated'

export default class LayerMeta {
    public geometryType: FlatGeobuf.GeometryType
    public columns: ColumnMeta[]
    constructor(geometryType: FlatGeobuf.GeometryType, columns: ColumnMeta[]) {
        this.geometryType = geometryType
        this.columns = columns
    }
}

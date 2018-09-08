import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

const typeMap: { [index: string]: FlatGeobuf.GeometryType } = {
    Point: GeometryType.POINT,
    MultiPoint: GeometryType.MULTIPOINT,
    LineString: GeometryType.LINESTRING,
    MultiLineString: GeometryType.MULTILINESTRING,
    Polygon: GeometryType.POLYGON,
    MultiPolygon: GeometryType.MULTIPOLYGON,
    GeometryCollection: GeometryType.GEOMETRYCOLLECTION,
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: any) {
    const type = typeMap[geometry.type]
    Geometry.startGeometry(builder)
    Geometry.addType(builder, type)

    // TODO: parse coordinates

    const i = Geometry.endGeometry(builder)
    builder.finish(i)
}
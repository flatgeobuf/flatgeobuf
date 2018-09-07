import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from './flatgeobuf'

export function fromGeoJson(geojson: any) {
    const builder = new flatbuffers.Builder(0)

    const Geometry = FlatGeobuf.Geometry
    const GeometryType = FlatGeobuf.GeometryType

    const typeMap: { [index: string]: FlatGeobuf.GeometryType } = {
        LineString: GeometryType.LINESTRING,
        Point: GeometryType.POINT,
    }

    const type = typeMap[geojson.type]

    Geometry.startGeometry(builder)
    Geometry.addType(builder, typeMap[geojson.type])
    const point = Geometry.endGeometry(builder)
    builder.finish(point)

    return builder.dataBuffer()
}

export function toGeoJson(buf: flatbuffers.ByteBuffer) {
    const geometry = FlatGeobuf.Geometry.getRootAsGeometry(buf)
    const type = geometry.type()
    return type
}

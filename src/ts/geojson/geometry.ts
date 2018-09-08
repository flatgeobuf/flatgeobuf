import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

const typeMap: { [index: string]: FlatGeobuf.GeometryType } = {
    Point: GeometryType.Point,
    MultiPoint: GeometryType.MultiPoint,
    LineString: GeometryType.LineString,
    MultiLineString: GeometryType.MultiLineString,
    Polygon: GeometryType.Polygon,
    MultiPolygon: GeometryType.MultiPolygon,
    GeometryCollection: GeometryType.GeometryCollection,
}

export function buildGeometry(builder: flatbuffers.Builder, geometry: any) {
    const type = typeMap[geometry.type]
    const coordsOffset = Geometry.createCoordsVector(builder, geometry.coordinates)
    Geometry.startGeometry(builder)
    Geometry.addCoords(builder, coordsOffset)
    Geometry.addType(builder, type)
    const offset = Geometry.endGeometry(builder)
    return offset
}

export function fromGeometry(geometry: FlatGeobuf.Geometry) {
    const type = GeometryType[geometry.type()]
    const coordinates = Array.from(geometry.coordsArray())
    return {
        type,
        coordinates,
    }
}

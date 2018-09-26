import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf } from '../flatgeobuf_generated'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

export function buildGeometry(builder: flatbuffers.Builder, geometry: any) {
    const coordsOffset = Geometry.createCoordsVector(builder, geometry.coordinates)
    Geometry.startGeometry(builder)
    Geometry.addCoords(builder, coordsOffset)
    const offset = Geometry.endGeometry(builder)
    return offset
}

export function fromGeometry(geometry: FlatGeobuf.Geometry, type: FlatGeobuf.GeometryType) {
    const coordinates = Array.from(geometry.coordsArray())
    return {
        type: GeometryType[type],
        coordinates,
    }
}

export function toGeometryType(name: string): FlatGeobuf.GeometryType {
    const type: FlatGeobuf.GeometryType = (GeometryType as any)[name]
    return type
}

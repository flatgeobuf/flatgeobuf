import { flatbuffers } from 'flatbuffers'
import { FlatGeobuf} from '../flatgeobuf_generated'

const Geometry = FlatGeobuf.Geometry
const GeometryType = FlatGeobuf.GeometryType

export function buildGeometry(builder: flatbuffers.Builder, geometry: any) {
    const type: FlatGeobuf.GeometryType = (GeometryType as any)[geometry.type]
    const coordsOffset = Geometry.createCoordsVector(builder, geometry.coordinates)
    Geometry.startGeometry(builder)
    Geometry.addCoords(builder, coordsOffset)
    //Geometry.addType(builder, type)
    const offset = Geometry.endGeometry(builder)
    return offset
}

export function fromGeometry(geometry: FlatGeobuf.Geometry) {
    const type = GeometryType.Point //GeometryType[geometry.type()]
    const coordinates = Array.from(geometry.coordsArray())
    return {
        type,
        coordinates,
    }
}

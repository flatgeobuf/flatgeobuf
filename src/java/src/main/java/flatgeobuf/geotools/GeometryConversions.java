package flatgeobuf.geotools;

import java.io.IOException;
import java.util.stream.Stream;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.Polygon;

import flatgeobuf.generated.*;

public class GeometryConversions {
    public static int serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry, int geometryType, int dimensions) throws IOException {
        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        if (dimensions == 4)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        else if (dimensions == 3)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        int coordsOffset = Geometry.createCoordsVector(builder, coords);
        int[] lengths = null;
        int[] ringLengths = null;
        if (geometryType == GeometryType.MultiLineString) {
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                lengths = new int[mls.getNumGeometries()];
                for (int i = 1; i < mls.getNumGeometries() + 1; i++)
                lengths[i] = mls.getGeometryN(i).getNumPoints() * dimensions;
            }
        } else if (geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            ringLengths = new int[p.getNumInteriorRing() + 1];
            ringLengths[0] = p.getExteriorRing().getNumGeometries() * dimensions;
            for (int i = 1; i < p.getNumInteriorRing() + 1; i++)
                ringLengths[i] = p.getInteriorRingN(i).getNumGeometries() * dimensions;
        } else if (geometryType == GeometryType.MultiPolygon) {
            throw new RuntimeException("Not implemented yet");
        }
        int lengthsOffset = 0;
        int ringLengthsOffset = 0;
        if (lengths != null)
            lengthsOffset = Geometry.createLengthsVector(builder, lengths);
        if (ringLengths != null)
            ringLengthsOffset = Geometry.createRingLengthsVector(builder, ringLengths);
        Geometry.startGeometry(builder);
        Geometry.addCoords(builder, coordsOffset);
        if (lengths != null)
            Geometry.addLengths(builder, lengthsOffset);
        if (ringLengths != null)
            Geometry.addRingLengths(builder, ringLengthsOffset);
        return Geometry.endGeometry(builder);
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Geometry geometry, int geometryType, int dimensions) {
        GeometryFactory factory = new GeometryFactory();
        int coordsLength = geometry.coordsLength();
        int dimLengths = coordsLength / dimensions;
        Coordinate[] coordinates = new Coordinate[dimLengths];
        int c = 0;
        for (int i = 0; i < coordsLength; i = i + dimensions)
            coordinates[c++] = new Coordinate(geometry.coords(i), geometry.coords(i + 1));
        switch (geometryType) {
            case GeometryType.Point:
                return factory.createPoint(coordinates[0]);
            case GeometryType.MultiPoint:
                return factory.createMultiPointFromCoords(coordinates);
            case GeometryType.LineString:
                return factory.createLineString(coordinates);
            case GeometryType.MultiLineString:
                return factory.createLineString(coordinates);
            case GeometryType.Polygon:
                return factory.createPolygon(coordinates);
            default:
                throw new RuntimeException("Unknown geometry type");
        }
    }
}
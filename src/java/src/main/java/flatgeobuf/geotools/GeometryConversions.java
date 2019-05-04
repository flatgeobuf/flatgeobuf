package flatgeobuf.geotools;

import java.io.IOException;
import java.util.stream.Stream;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.Polygon;

import flatgeobuf.generated.*;

public class GeometryConversions {
    public static int write(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry, byte geometryType, byte dimensions) throws IOException {
        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        if (dimensions == 4)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        else if (dimensions == 3)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        int coordsOffset = Geometry.createCoordsVector(builder, coords);
        int ringLengthsOffset = 0;
        if (geometryType == GeometryType.Polygon) {
            Polygon polygon = (Polygon) geometry;
            List<Integer> ringLengthsList = new ArrayList<Integer>();
            ringLengthsList.add(polygon.getExteriorRing().getNumGeometries() * dimensions);
            for (int i = 0; i < polygon.getNumInteriorRing(); i++)
                ringLengthsList.add(polygon.getInteriorRingN(i).getNumGeometries() * dimensions);
            int[] ringLengths = ringLengthsList.stream().mapToInt(i -> i).toArray();
            ringLengthsOffset = Geometry.createRingLengthsVector(builder, ringLengths);
        }
            
        Geometry.startGeometry(builder);
        Geometry.addCoords(builder, coordsOffset);
        Geometry.addRingLengths(builder, ringLengthsOffset);
        return Geometry.endGeometry(builder);
    }
}
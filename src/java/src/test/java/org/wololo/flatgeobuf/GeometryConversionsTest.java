package org.wololo.flatgeobuf;

import com.google.flatbuffers.FlatBufferBuilder;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;
import org.locationtech.jts.geom.MultiPoint;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.CoordinateSequence;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
import org.locationtech.jts.io.WKTWriter;
import org.wololo.flatgeobuf.generated.GeometryType;

import java.io.IOException;
import java.util.Date;

import static org.junit.Assert.assertEquals;

public class GeometryConversionsTest {

    @Test
    public void toGeometryType() {
        assertEquals(GeometryType.Unknown, GeometryConversions.toGeometryType(Geometry.class));
        assertEquals(GeometryType.Point, GeometryConversions.toGeometryType(Point.class));
        assertEquals(GeometryType.LineString, GeometryConversions.toGeometryType(LineString.class));
        assertEquals(GeometryType.Polygon, GeometryConversions.toGeometryType(Polygon.class));
        assertEquals(GeometryType.MultiPoint, GeometryConversions.toGeometryType(MultiPoint.class));
        assertEquals(GeometryType.MultiLineString, GeometryConversions.toGeometryType(MultiLineString.class));
        assertEquals(GeometryType.MultiPolygon, GeometryConversions.toGeometryType(MultiPolygon.class));
    }

    @Test(expected = RuntimeException.class)
    public void unknowGeometryShouldThrow() {
        GeometryConversions.toGeometryType(Date.class);
    }

    @Test
    public void toGeometryTypeWithGeometrySubClass() {
        assertEquals(GeometryType.Point, GeometryConversions.toGeometryType(MyPoint.class));
    }


    private static String serializeDeserializeRound(String ewkt) throws ParseException, IOException {
        Geometry geom = new WKTReader().read(ewkt);
        FlatBufferBuilder flatBufferBuilder = new FlatBufferBuilder();
        byte GeomType = GeometryConversions.toGeometryType(geom.getClass());
        int geometryOffset = GeometryConversions.serialize(flatBufferBuilder, geom, GeomType);
        flatBufferBuilder.finish(geometryOffset);
        org.wololo.flatgeobuf.generated.Geometry fgbGeom = org.wololo.flatgeobuf.generated.Geometry.
                getRootAsGeometry(flatBufferBuilder.dataBuffer());
        Geometry geomJTSOutput = GeometryConversions.deserialize(fgbGeom, GeomType);
        if(geomJTSOutput == null) {
            throw new IOException("Null geometry");
        }
        try {
            return new WKTWriter(4).write(geomJTSOutput);
        } catch (IllegalArgumentException ex) {
            throw new IOException("Can't ewkb JTS geometry:\n"+new WKTWriter(4).write(geomJTSOutput), ex);
        }
    }

    @Test
    public void testXYZ() throws IOException, ParseException {
        String expectedWKT = "POINT Z(3 5 7)";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "LINESTRING Z(3 5 7, 4 8 8, 6 9 10)";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "POLYGON Z((10 5 1, 10 10 2, 8 10 3, 8 5 4, 10 5 1))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTIPOINT Z((3 5 1), (4 8 2), (6 9 3))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTILINESTRING Z((3 5 1, 4 8 1, 6 9 2), (9 2 9, 1 2 8, 6 6 7), (10 1 2, 9 2 1, 8 3 3))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTIPOLYGON Z(((10 5 1, 10 10 2, 8 10 3, 8 5 4, 10 5 1)), ((5 5 1, 5 5 2, 4 5 3, 4 2 4, 5 5 1)))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));
    }

    @Test
    public void testXYZM() throws IOException, ParseException {
        String expectedWKT = "POINT ZM(3 5 7 8)";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "LINESTRING ZM(3 5 7 4, 4 8 8 6, 6 9 10 8)";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "POLYGON ZM((10 5 1 5, 10 10 2 4, 8 10 3 7, 8 5 4 2, 10 5 1 5))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTIPOINT ZM((3 5 1 8), (4 8 2 12), (6 9 3 44))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTILINESTRING ZM((3 5 1 4, 4 8 1 8, 6 9 2 12), (9 2 9 44, 1 2 8 55, 6 6 7 5), (10 1 2 6, 9 2 1 9, 8 3 3 12))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));

        expectedWKT = "MULTIPOLYGON ZM(((10 5 1 1, 10 10 2 66, 8 10 3 55, 8 5 4 4, 10 5 1 1)), ((5 5 1 5, 5 5 2 42, 4 5 3 41, 4 2 4 4, 5 5 1 5)))";
        assertEquals(expectedWKT, serializeDeserializeRound(expectedWKT));
    }


    private static class MyPoint extends Point {
        public MyPoint(CoordinateSequence coordinates, GeometryFactory factory) {
            super(coordinates, factory);
        }
    }


}

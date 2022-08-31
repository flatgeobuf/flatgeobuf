package org.wololo.flatgeobuf;

import static org.junit.Assert.assertEquals;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

import com.google.flatbuffers.FlatBufferBuilder;

import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
import org.locationtech.jts.io.WKTWriter;
import org.wololo.flatgeobuf.GeometryConversions;

public class GeometryRoundtripTest {

    String roundTrip(String wkt) throws IOException {
        WKTReader reader = new WKTReader();
        Geometry geometry;
        try {
            geometry = reader.read(wkt);
        } catch (ParseException e) {
            throw new RuntimeException(e);
        }
        byte geometryType = GeometryConversions.toGeometryType(geometry.getClass());
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);
        int gometryOffset = GeometryConversions.serialize(builder, geometry, geometryType);
        builder.finish(gometryOffset);
        byte[] bytes = builder.sizedByteArray();
        ByteBuffer bb = ByteBuffer.wrap(bytes);
        bb.order(ByteOrder.LITTLE_ENDIAN);
        GeometryConversions.deserialize(org.wololo.flatgeobuf.generated.Geometry.getRootAsGeometry(bb), geometryType);
        WKTWriter writer = new WKTWriter();
        return writer.write(geometry).replace('−', '-');
    }

    @Test
    public void point() throws IOException {
        String expected = "POINT (1.2 -2.1)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void pointEmpty() throws IOException {
        String expected = "POINT EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multipoint() throws IOException {
        String expected = "MULTIPOINT ((10 40), (40 30), (20 20), (30 10))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multipointEmpty() throws IOException {
        String expected = "MULTIPOINT EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void linestring() throws IOException {
        String expected = "LINESTRING (1.2 -2.1, 2.4 -4.8)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void linestringEmpty() throws IOException {
        String expected = "LINESTRING EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multilinestring() throws IOException {
        String expected = "MULTILINESTRING ((1 2, 3 4, 5 6), (7 8, 9 10, 11 12, 13 14))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multilinestringEmpty() throws IOException {
        String expected = "MULTILINESTRING EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void polygon() throws IOException {
        String expected = "POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void polygon_hole() throws IOException {
        String expected = "POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void polygonEmpty() throws IOException {
        String expected = "POLYGON EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multipolygon_single() throws IOException {
        String expected = "MULTIPOLYGON (((30 10, 40 40, 20 40, 10 20, 30 10)))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multipolygon() throws IOException {
        String expected = "MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)), ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multipolygonEmpty() throws IOException {
        String expected = "MULTIPOLYGON EMPTY";
        assertEquals(expected, roundTrip(expected));
    }
}

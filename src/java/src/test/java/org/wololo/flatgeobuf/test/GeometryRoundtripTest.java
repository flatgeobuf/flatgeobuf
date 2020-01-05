package org.wololo.flatgeobuf.test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertArrayEquals;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;

import org.geotools.data.memory.MemoryFeatureCollection;
import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.FeatureIterator;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.geotools.geojson.feature.FeatureJSON;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
import org.locationtech.jts.io.WKTWriter;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;

import org.wololo.flatgeobuf.geotools.FeatureCollectionConversions;

public class GeometryRoundtripTest {

    SimpleFeatureCollection makeFCFromGeoJSON(String geojson) {
        FeatureJSON featureJSON = new FeatureJSON();
        SimpleFeatureCollection fc;
        try {
            fc = (SimpleFeatureCollection) featureJSON
                    .readFeatureCollection(new ByteArrayInputStream(geojson.getBytes(StandardCharsets.UTF_8)));
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        return fc;
    }

    SimpleFeatureCollection makeFC(String wkt) {
        WKTReader reader = new WKTReader();
        Geometry geometry;
        try {
            geometry = reader.read(wkt);
        } catch (ParseException e) {
            throw new RuntimeException(e);
        }
        SimpleFeatureTypeBuilder ftb = new SimpleFeatureTypeBuilder();
        ftb.setName("testType");
        ftb.add("geometryProperty", geometry.getClass());
        SimpleFeatureType ft = ftb.buildFeatureType();
        SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
        fb.add(geometry);
        SimpleFeature f = fb.buildFeature("fid");
        MemoryFeatureCollection fc = new MemoryFeatureCollection(ft);
        fc.add(f);
        return fc;
    }

    SimpleFeatureCollection makeFC(String[] wkts, Class<?> geometryClass) {
        WKTReader reader = new WKTReader();
        Geometry geometry;
        SimpleFeatureTypeBuilder ftb = new SimpleFeatureTypeBuilder();
        ftb.setName("testType");
        ftb.add("geometryProperty", geometryClass);
        SimpleFeatureType ft = ftb.buildFeatureType();
        MemoryFeatureCollection fc = new MemoryFeatureCollection(ft);
        for (int i = 0; i < wkts.length; i++) {
            SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
            try {
                geometry = reader.read(wkts[i]);
            } catch (ParseException e) {
                throw new RuntimeException(e);
            }
            fb.add(geometry);
            SimpleFeature f = fb.buildFeature(Integer.toString(i));
            fc.add(f);
        }
        return fc;
    }

    String roundTrip(String wkt) throws IOException {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        FeatureCollectionConversions.serialize(makeFC(wkt), 1, os);
        ByteBuffer bb = ByteBuffer.wrap(os.toByteArray());
        bb.order(ByteOrder.LITTLE_ENDIAN);
        SimpleFeatureCollection fc = FeatureCollectionConversions.deserialize(bb);
        Geometry geometry = (Geometry) fc.features().next().getDefaultGeometry();
        WKTWriter writer = new WKTWriter(4);
        return writer.write(geometry).replace('−', '-');
    }

    @Test
    public void point() throws IOException {
        String expected = "POINT (1.2 -2.1)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void pointZ() throws IOException {
        String expected = "POINT Z(1.2 -2.1 1)";
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
    public void linestringZ() throws IOException {
        String expected = "LINESTRING Z(1.2 -2.1 1, 2.4 -4.8 2)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void linestringEmpty() throws IOException {
        String expected = "LINESTRING EMPTY";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void multilinestring() throws IOException {
        String expected = "MULTILINESTRING ((10 10, 20 20, 10 40), (40 40, 30 30, 40 20, 30 10))";
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
    public void polygonZ() throws IOException {
        String expected = "POLYGON Z((30 10 1, 40 40 1, 20 40 1, 10 20 1, 30 10 1))";
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

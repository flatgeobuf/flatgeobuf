package flatgeobuf.test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertArrayEquals;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

import org.geotools.data.memory.MemoryFeatureCollection;
import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.FeatureIterator;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
import org.locationtech.jts.io.WKTWriter;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;

import flatgeobuf.geotools.FeatureCollectionConversions;

public class GeometryRoundtripTest {
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
            boolean result = fc.add(f);
            System.out.println(result);
        }
        return fc;
    }

    String roundTrip(String wkt) throws IOException {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        FeatureCollectionConversions.serialize(makeFC(wkt), os);
        ByteBuffer bb = ByteBuffer.wrap(os.toByteArray());
        bb.order(ByteOrder.LITTLE_ENDIAN);
        SimpleFeatureCollection fc = FeatureCollectionConversions.deserialize(bb);
        Geometry geometry = (Geometry) fc.features().next().getDefaultGeometry();
        WKTWriter writer = new WKTWriter();
        return writer.write(geometry);
    }

    String[] roundTrip(String[] wkts, Class<?> geometryClass) throws IOException {
        String[] newWkts = new String[wkts.length];
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        FeatureCollectionConversions.serialize(makeFC(wkts, geometryClass), os);
        ByteBuffer bb = ByteBuffer.wrap(os.toByteArray());
        bb.order(ByteOrder.LITTLE_ENDIAN);
        SimpleFeatureCollection fc = FeatureCollectionConversions.deserialize(bb);
        WKTWriter writer = new WKTWriter();
        int c = 0;
        try (FeatureIterator<SimpleFeature> iterator = fc.features()) {
            while (iterator.hasNext()) {
                SimpleFeature feature = iterator.next();
                Geometry geometry = (Geometry) feature.getDefaultGeometry();
                newWkts[c++] = writer.write(geometry);
            }
        }
        return newWkts;
    }

    @Test
    public void point() throws IOException
    {
        String expected = "POINT (1.2 -2.1)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void points() throws IOException
    {
        String[] expected = new String[] { "POINT (1.2 -2.1)", "POINT (10.2 -20.1)" };
        assertArrayEquals(expected, roundTrip(expected, Point.class));
    }

    @Test
    public void multipoint() throws IOException
    {
        String expected = "MULTIPOINT ((10 40), (40 30), (20 20), (30 10))";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void linestring() throws IOException
    {
        String expected = "LINESTRING (1.2 -2.1, 2.4 -4.8)";
        assertEquals(expected, roundTrip(expected));
    }

    @Test
    public void polygon() throws IOException
    {
        String expected = "POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))";
        assertEquals(expected, roundTrip(expected));
    }
}

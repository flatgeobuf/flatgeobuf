package flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;

import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.DefaultFeatureCollection;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
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
        DefaultFeatureCollection fc = new DefaultFeatureCollection();
        fc.add(f);
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

    @Test
    public void point() throws IOException
    {
        String expected = "POINT (1.2 -2.1)";
        assertEquals(expected, roundTrip(expected));
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

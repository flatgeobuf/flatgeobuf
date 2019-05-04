package flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.ByteArrayOutputStream;
import java.io.IOException;

import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.DefaultFeatureCollection;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
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

    @Test
    public void point() throws IOException
    {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        SimpleFeatureCollection fc = makeFC("POINT(1.2 -2.1)");
        FeatureCollectionConversions.write(fc, os);
        int size = os.size();
        assertEquals(80, size);
    }

    @Test
    public void multipoint() throws IOException
    {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        SimpleFeatureCollection fc = makeFC("MULTIPOINT(10 40, 40 30, 20 20, 30 10)");
        FeatureCollectionConversions.write(fc, os);
        int size = os.size();
        assertEquals(140, size);
    }

    @Test
    public void linestring() throws IOException
    {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        SimpleFeatureCollection fc = makeFC("LINESTRING(1.2 -2.1, 2.4 -4.8)");
        FeatureCollectionConversions.write(fc, os);
        int size = os.size();
        assertEquals(108, size);
    }

    @Test
    public void polygon() throws IOException
    {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        SimpleFeatureCollection fc = makeFC("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))");
        FeatureCollectionConversions.write(fc, os);
        int size = os.size();
        assertEquals(172, size);
    }
}

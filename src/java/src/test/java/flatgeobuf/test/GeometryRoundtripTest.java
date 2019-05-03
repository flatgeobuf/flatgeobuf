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
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.io.ParseException;
import org.locationtech.jts.io.WKTReader;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;

import flatgeobuf.geotools.FeatureCollectionConversions;

public class GeometryRoundtripTest {
    SimpleFeatureCollection makeFC(String wkt) {
        SimpleFeatureTypeBuilder ftb = new SimpleFeatureTypeBuilder();
        ftb.setName("testType");
        ftb.add("geometryProperty", Point.class);
        SimpleFeatureType ft = ftb.buildFeatureType();
        SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
        WKTReader reader = new WKTReader();
        Geometry geometry;
        try {
            geometry = reader.read(wkt);
        } catch (ParseException e) {
            throw new RuntimeException(e);
        }
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
        assertEquals(16, size);
    }
}

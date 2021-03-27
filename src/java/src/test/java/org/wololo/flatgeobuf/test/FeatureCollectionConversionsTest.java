package org.wololo.flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.File;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.Iterator;

import org.junit.Test;
import org.locationtech.jts.geom.Envelope;
import org.opengis.feature.simple.SimpleFeature;
import org.wololo.flatgeobuf.geotools.FeatureCollectionConversions;

public class FeatureCollectionConversionsTest {
    @Test
    public void countriesTest() throws IOException {
        File file = new File("../../test/data/countries.fgb");
        byte[] bytes = Files.readAllBytes(file.toPath());
        ByteBuffer bb = ByteBuffer.wrap(bytes);
        bb.order(ByteOrder.LITTLE_ENDIAN);

        Iterator<SimpleFeature> it = FeatureCollectionConversions.deserialize(bb, null).iterator();
        int count = 0;
        while (it.hasNext()) {
            it.next();
            count++;
        }
        assertEquals(179, count);
    }

    @Test
    public void countriesTestFilter() throws IOException {
        File file = new File("../../test/data/countries.fgb");
        byte[] bytes = Files.readAllBytes(file.toPath());
        ByteBuffer bb = ByteBuffer.wrap(bytes);
        bb.order(ByteOrder.LITTLE_ENDIAN);

        Envelope rect = new Envelope(12, 12, 56, 56);

        Iterator<SimpleFeature> it = FeatureCollectionConversions.deserialize(bb, rect).iterator();
        int count = 0;
        while (it.hasNext()) {
            it.next();
            count++;
        }
        assertEquals(3, count);
    }
}

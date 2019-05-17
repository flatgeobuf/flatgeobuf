package flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UnsupportedEncodingException;
import java.net.URISyntaxException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.geojson.feature.FeatureJSON;
import org.junit.Test;

import flatgeobuf.geotools.FeatureCollectionConversions;

public class AttributeRoundtripTest {

    SimpleFeatureCollection makeFC(String geojson) {
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

    String roundTrip(String geojson) throws IOException {
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        FeatureCollectionConversions.serialize(makeFC(geojson), 1, os);
        ByteBuffer bb = ByteBuffer.wrap(os.toByteArray());
        bb.order(ByteOrder.LITTLE_ENDIAN);
        SimpleFeatureCollection fc = FeatureCollectionConversions.deserialize(bb);
        FeatureJSON featureJSON = new FeatureJSON();
        os = new ByteArrayOutputStream();
        featureJSON.writeFeatureCollection(fc, os);
        return os.toString(StandardCharsets.UTF_8.name());
    }

    String getResource(String name) throws URISyntaxException, UnsupportedEncodingException, IOException {
        Path resourcePath = Paths.get(this.getClass().getResource(name).toURI());
        String resource = new String(Files.readAllBytes(resourcePath), StandardCharsets.UTF_8.name());
        FeatureJSON featureJSON = new FeatureJSON();
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        featureJSON.writeFeatureCollection(featureJSON.readFeatureCollection(resource), os);
        return os.toString(StandardCharsets.UTF_8.name());
    }

    @Test
    public void point() throws IOException, URISyntaxException {
        String expected = getResource("1.json");
        System.out.println("IN: " + expected);
        assertEquals(expected, roundTrip(expected));
    }

}

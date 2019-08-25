package org.wololo.flatgeobuf.test;

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
import org.json.*;

import org.wololo.flatgeobuf.geotools.FeatureCollectionConversions;

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

    String removeId(String json) {
        JSONObject jsonObject = new JSONObject(json);
        for (Object feature : jsonObject.getJSONArray("features"))
            ((JSONObject)feature).remove("id");
        return jsonObject.toString(1);
    }

    @Test
    public void mixed1() throws IOException, URISyntaxException {
        String expected = removeId(getResource("1.json"));
        String actual = removeId(roundTrip(expected));
        assertEquals(expected, actual);
    }

    public void mixed2() throws IOException, URISyntaxException {
        String expected = removeId(getResource("2.json"));
        String actual = removeId(roundTrip(expected));
        assertEquals(expected, actual);
    }

}

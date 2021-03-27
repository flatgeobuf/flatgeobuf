package org.wololo.flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UnsupportedEncodingException;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.net.URISyntaxException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.LocalTime;

import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.data.memory.MemoryFeatureCollection;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.geotools.geojson.feature.FeatureJSON;
import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.json.*;

import org.wololo.flatgeobuf.geotools.FeatureCollectionConversions;
import org.wololo.flatgeobuf.geotools.FeatureTypeConversions;
import org.wololo.flatgeobuf.geotools.HeaderMeta;

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
        HeaderMeta headerMeta = FeatureTypeConversions.deserialize(bb);
        SimpleFeatureType featureType = FeatureTypeConversions.getSimpleFeatureType(headerMeta, "unknown");
        SimpleFeatureCollection fc = FeatureCollectionConversions.deserialize(bb, headerMeta, featureType);
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

    @Test
    public void mixed2() throws IOException, URISyntaxException {
        String expected = removeId(getResource("2.json"));
        String actual = removeId(roundTrip(expected));
        assertEquals(expected, actual);
    }

    @Test
    public void exotic1() throws IOException {
        SimpleFeatureTypeBuilder sftb = new SimpleFeatureTypeBuilder();
        sftb.setName("testName");
        sftb.add("geometryPropertyName", Geometry.class);
        sftb.add("exotic1_1", Integer.class);
        sftb.add("exotic1_2", BigInteger.class);
        sftb.add("exotic1_3", LocalDateTime.class);
        sftb.add("exotic1_4", LocalDate.class);
        sftb.add("exotic1_5", LocalTime.class);
        sftb.add("exotic1_6", BigDecimal.class);
        sftb.add("exotic1_7", Byte.class);
        sftb.add("exotic1_8", Short.class);
        SimpleFeatureType ft = sftb.buildFeatureType();
        SimpleFeatureBuilder sfb = new SimpleFeatureBuilder(ft);
        sfb.set("exotic1_1", new Integer("99"));
        sfb.set("exotic1_2", new BigInteger("1111111111111111111"));
        sfb.set("exotic1_3", LocalDateTime.now());
        sfb.set("exotic1_4", LocalDate.now());
        sfb.set("exotic1_5", LocalTime.now());
        sfb.set("exotic1_6", new BigDecimal("1.1111"));
        sfb.set("exotic1_7", new Byte("99"));
        sfb.set("exotic1_8", new Short("9999"));
        SimpleFeature sf = sfb.buildFeature("0");
        MemoryFeatureCollection expected = new MemoryFeatureCollection(ft);
        expected.add(sf);
        ByteArrayOutputStream os = new ByteArrayOutputStream();
        FeatureCollectionConversions.serialize(expected, 0, os);
        ByteBuffer bb = ByteBuffer.wrap(os.toByteArray());
        bb.order(ByteOrder.LITTLE_ENDIAN);
        SimpleFeatureCollection actual = FeatureCollectionConversions.deserialize(bb);
        SimpleFeature expectedFeature = (SimpleFeature) expected.toArray()[0];
        SimpleFeature actualFeature = (SimpleFeature) actual.toArray()[0];
        //assertEquals(expectedFeature.getAttribute(0).toString(), actualFeature.getAttribute(0).toString());
        assertEquals(expectedFeature.getAttribute(1).toString(), actualFeature.getAttribute(1).toString());
        assertEquals(expectedFeature.getAttribute(2).toString(), actualFeature.getAttribute(2).toString());
        assertEquals(expectedFeature.getAttribute(3).toString(), actualFeature.getAttribute(3).toString());
        assertEquals(expectedFeature.getAttribute(4).toString(), actualFeature.getAttribute(4).toString());
        assertEquals(expectedFeature.getAttribute(5).toString(), actualFeature.getAttribute(5).toString());
        assertEquals(expectedFeature.getAttribute(6).toString(), actualFeature.getAttribute(6).toString());
        assertEquals(expectedFeature.getAttribute(7).toString(), actualFeature.getAttribute(7).toString());
        assertEquals(expectedFeature.getAttribute(8).toString(), actualFeature.getAttribute(8).toString());
    }

}

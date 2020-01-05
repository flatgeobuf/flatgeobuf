package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.ByteBuffer;

import com.google.flatbuffers.ByteBufferUtil;
import static com.google.flatbuffers.Constants.SIZE_PREFIX_LENGTH;

import org.wololo.flatgeobuf.generated.Feature;

import org.geotools.data.memory.MemoryFeatureCollection;
import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.FeatureIterator;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;

public class FeatureCollectionConversions {

    public static void serialize(SimpleFeatureCollection featureCollection, long featuresCount,
            OutputStream outputStream) throws IOException {
        if (featuresCount == 0)
            return;

        SimpleFeatureType featureType = featureCollection.getSchema();
        HeaderMeta headerMeta = null;
        try (FeatureIterator<SimpleFeature> iterator = featureCollection.features()) {
            while (iterator.hasNext()) {
                SimpleFeature feature = iterator.next();
                if (headerMeta == null)
                    headerMeta = FeatureTypeConversions.serialize(featureType, feature, featuresCount, outputStream);
                byte[] featureBuffer = FeatureConversions.serialize(feature, headerMeta);
                outputStream.write(featureBuffer);
            }
        }
    }

    public static SimpleFeatureCollection deserialize(ByteBuffer bb) throws IOException {
        HeaderMeta headerMeta = FeatureTypeConversions.deserialize(bb, "testName", "geometryPropertyName");
        int offset = headerMeta.offset;
        SimpleFeatureType ft = headerMeta.featureType;
        SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
        MemoryFeatureCollection fc = new MemoryFeatureCollection(ft);
        while (bb.hasRemaining()) {
            int featureSize = ByteBufferUtil.getSizePrefix(bb);
            bb.position(offset += SIZE_PREFIX_LENGTH);
            Feature feature = Feature.getRootAsFeature(bb);
            bb.position(offset += featureSize);
            SimpleFeature f = FeatureConversions.deserialize(feature, fb, headerMeta);
            fc.add(f);
        }
        return fc;
    }

}
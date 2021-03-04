package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.ByteBuffer;

import com.google.flatbuffers.ByteBufferUtil;
import com.google.flatbuffers.FlatBufferBuilder;
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

        SimpleFeatureType featureType = featureCollection.getSchema();

        FlatBufferBuilder builder = FlatBuffers.newBuilder(16 * 1024);
        try {
            HeaderMeta headerMeta =
                    FeatureTypeConversions.serialize(featureType, featuresCount, outputStream, builder);
            builder.clear();
            
            try (FeatureIterator<SimpleFeature> iterator = featureCollection.features()) {
                while (iterator.hasNext()) {
                    SimpleFeature feature = iterator.next();
                    FeatureConversions.serialize(feature, headerMeta, outputStream, builder);
                    builder.clear();
                }
            }
        } finally {
            FlatBuffers.release(builder);
        }
    }

    public static SimpleFeatureCollection deserialize(ByteBuffer bb) throws IOException {
        HeaderMeta headerMeta = FeatureTypeConversions.deserialize(bb, "testName", "geometryPropertyName");
        int offset = headerMeta.offset;
        SimpleFeatureType ft = headerMeta.featureType;
        SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
        MemoryFeatureCollection fc = new MemoryFeatureCollection(ft);
        long count = 0;
        while (bb.hasRemaining()) {
            int featureSize = ByteBufferUtil.getSizePrefix(bb);
            bb.position(offset += SIZE_PREFIX_LENGTH);
            Feature feature = Feature.getRootAsFeature(bb);
            bb.position(offset += featureSize);
            SimpleFeature f = FeatureConversions.deserialize(feature, fb, headerMeta, Long.toString(count++));
            fc.add(f);
        }
        return fc;
    }

}
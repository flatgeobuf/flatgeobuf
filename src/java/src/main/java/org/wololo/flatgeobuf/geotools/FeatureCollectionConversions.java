package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Iterator;

import com.google.flatbuffers.ByteBufferUtil;
import com.google.flatbuffers.FlatBufferBuilder;
import static com.google.flatbuffers.Constants.SIZE_PREFIX_LENGTH;

import org.wololo.flatgeobuf.PackedRTree;
import org.wololo.flatgeobuf.PackedRTree.SearchHit;
import org.wololo.flatgeobuf.generated.Feature;

import org.geotools.data.memory.MemoryFeatureCollection;
import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.FeatureIterator;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.locationtech.jts.geom.Envelope;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;

public class FeatureCollectionConversions {

    private static final class ReadHitsIterable implements Iterable<SimpleFeature> {
        private final SimpleFeatureBuilder fb;
        private final ArrayList<SearchHit> hits;
        private final HeaderMeta headerMeta;
        private final int featuresOffset;
        private final ByteBuffer bb;

        private ReadHitsIterable(SimpleFeatureBuilder fb, ArrayList<SearchHit> hits, HeaderMeta headerMeta, int featuresOffset,
                ByteBuffer bb) {
            this.fb = fb;
            this.hits = hits;
            this.headerMeta = headerMeta;
            this.featuresOffset = featuresOffset;
            this.bb = bb;
        }

        @Override
        public Iterator<SimpleFeature> iterator() {
            Iterator<SimpleFeature> it = new Iterator<SimpleFeature>() {
                int count = 0;
                @Override
                public boolean hasNext() {
                    return count < hits.size();
                }
                @Override
                public SimpleFeature next() {
                    SearchHit hit = hits.get(count);
                    int offset = featuresOffset + (int) hit.offset;
                    bb.position(offset);
                    int featureSize = ByteBufferUtil.getSizePrefix(bb);
                    bb.position(offset += SIZE_PREFIX_LENGTH);
                    Feature feature = Feature.getRootAsFeature(bb);
                    bb.position(offset += featureSize);
                    SimpleFeature f = FeatureConversions.deserialize(feature, fb, headerMeta, Long.toString(count++));
                    return f;
                }
            };
            return it;
        }
    }

    private static final class ReadAllInterable implements Iterable<SimpleFeature> {
        private final HeaderMeta headerMeta;
        private final int featuresOffset;
        private final ByteBuffer bb;
        private final SimpleFeatureBuilder fb;

        private ReadAllInterable(HeaderMeta headerMeta, int featuresOffset, ByteBuffer bb,
                SimpleFeatureBuilder fb) {
            this.headerMeta = headerMeta;
            this.featuresOffset = featuresOffset;
            this.bb = bb;
            this.fb = fb;
        }

        @Override
        public Iterator<SimpleFeature> iterator() {
            Iterator<SimpleFeature> it = new Iterator<SimpleFeature>() {
                int count = 0;
                int offset = featuresOffset;
                @Override
                public boolean hasNext() {
                    return bb.hasRemaining();
                }
                @Override
                public SimpleFeature next() {
                    int featureSize = ByteBufferUtil.getSizePrefix(bb);
                    bb.position(offset += SIZE_PREFIX_LENGTH);
                    Feature feature = Feature.getRootAsFeature(bb);
                    bb.position(offset += featureSize);
                    SimpleFeature f = FeatureConversions.deserialize(feature, fb, headerMeta, Long.toString(count++));
                    return f;
                }
            };
            return it;
        }
    }

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
        if (headerMeta.featuresCount > 0 && headerMeta.indexNodeSize > 0) {
            offset += PackedRTree.calcSize((int) headerMeta.featuresCount, headerMeta.indexNodeSize);
            bb.position(offset);
        }
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

    public static Iterable<SimpleFeature> deserialize(ByteBuffer bb, Envelope rect) throws IOException {
        HeaderMeta headerMeta = FeatureTypeConversions.deserialize(bb, "testName", "geometryPropertyName");
        int treeSize = headerMeta.featuresCount > 0 && headerMeta.indexNodeSize > 0 ? (int) PackedRTree.calcSize((int) headerMeta.featuresCount, headerMeta.indexNodeSize) : 0;
        int featuresOffset = headerMeta.offset + treeSize;
        SimpleFeatureType ft = headerMeta.featureType;
        SimpleFeatureBuilder fb = new SimpleFeatureBuilder(ft);
        if (treeSize > 0)
            bb.position(featuresOffset);

        Iterable<SimpleFeature> iterable;
        if (rect == null) {
            iterable = new ReadAllInterable(headerMeta, featuresOffset, bb, fb);
        } else {
            ArrayList<SearchHit> hits = new PackedRTree().search(bb, headerMeta.offset, (int) headerMeta.featuresCount, headerMeta.indexNodeSize, rect);
            iterable = new ReadHitsIterable(fb, hits, headerMeta, featuresOffset, bb);
        }

        return iterable;
    }

}
package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

import com.google.flatbuffers.FlatBufferBuilder;

import org.wololo.flatgeobuf.generated.ColumnType;
import org.wololo.flatgeobuf.generated.Feature;
import org.wololo.flatgeobuf.generated.Geometry;

import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.opengis.feature.simple.SimpleFeature;

public class FeatureConversions {

    private static void writeString(ByteBuffer bb, String value) {
        byte[] stringBytes = ((String) value).getBytes(StandardCharsets.UTF_8);
        bb.putInt(stringBytes.length);
        bb.put(stringBytes);
    }

    public static byte[] serialize(SimpleFeature feature, HeaderMeta headerMeta) throws IOException {
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);
        org.locationtech.jts.geom.Geometry geometry = (org.locationtech.jts.geom.Geometry) feature.getDefaultGeometry();

        ByteBuffer bb = ByteBuffer.allocate(1024 * 1024);
        bb.order(ByteOrder.LITTLE_ENDIAN);
        for (short i = 0; i < headerMeta.columns.size(); i++) {
            ColumnMeta column = headerMeta.columns.get(i);
            byte type = column.type;
            Object value = feature.getAttribute(column.name);
            if (value == null)
                continue;
            bb.putShort(i);
            if (type == ColumnType.Bool)
                bb.put((byte) ((boolean)value ? 1 : 0));
            else if (type == ColumnType.Int)
                bb.putInt((int) value);
            else if (type == ColumnType.Long)
                bb.putLong((long) value);
            else if (type == ColumnType.Double)
                bb.putDouble((double) value);
            else if (type == ColumnType.String)
                writeString(bb, (String) value);
            else
                throw new RuntimeException("Unknown type " + type);
        }

        int propertiesOffset = 0;
        if (bb.position() > 0) {
            byte[] data = Arrays.copyOfRange(bb.array(), 0, bb.position());
            propertiesOffset = Feature.createPropertiesVector(builder, data);
        }
        GeometryOffsets go = GeometryConversions.serialize(builder, geometry, headerMeta.geometryType, headerMeta);
        int geometryOffset;
        if (go.gos != null) {
            int[] partOffsets = new int[go.gos.length];
            for (int i = 0; i < go.gos.length; i++) {
                GeometryOffsets goPart = go.gos[i];
                int partOffset = Geometry.createGeometry(builder, goPart.endsOffset,goPart.xyOffset, goPart.zOffset, goPart.mOffset, 0, 0, 0, 0);
                partOffsets[i] = partOffset;
            }
            int partsOffset = Geometry.createPartsVector(builder, partOffsets);
            geometryOffset = Geometry.createGeometry(builder, 0, 0, 0, 0, 0, 0, 0, partsOffset);
        } else {
            geometryOffset = Geometry.createGeometry(builder, go.endsOffset, go.xyOffset, go.zOffset, go.mOffset, 0, 0, 0, 0);
        }
        int featureOffset = Feature.createFeature(builder, geometryOffset, propertiesOffset, 0);
        builder.finishSizePrefixed(featureOffset);

        return builder.sizedByteArray();
    }

    private static void readString(ByteBuffer bb, SimpleFeatureBuilder fb, String name) {
        int length = bb.getInt();
        byte[] stringBytes = new byte[length];
        bb.get(stringBytes, 0, length);
        String value = new String(stringBytes, StandardCharsets.UTF_8);
        fb.set(name, value);
    }

    public static SimpleFeature deserialize(Feature feature, SimpleFeatureBuilder fb, HeaderMeta headerMeta) {
        Geometry geometry = feature.geometry();
        fb.add(GeometryConversions.deserialize(geometry, headerMeta.geometryType, headerMeta));
        int propertiesLength = feature.propertiesLength();
        if (propertiesLength > 0) {
            ByteBuffer bb = feature.propertiesAsByteBuffer();
            while (bb.hasRemaining()) {
                short i = bb.getShort();
                ColumnMeta columnMeta = headerMeta.columns.get(i);
                String name = columnMeta.name;
                byte type = columnMeta.type;
                if (type == ColumnType.Bool)
                    fb.set(name, bb.get() > 0 ? true : false);
                else if (type == ColumnType.Int)
                    fb.set(name, bb.getInt());
                else if (type == ColumnType.Long)
                    fb.set(name, bb.getLong());
                else if (type == ColumnType.Double)
                    fb.set(name, bb.getDouble());
                else if (type == ColumnType.String)
                    readString(bb, fb, name);
                else
                    throw new RuntimeException("Unknown type");
            }
        }
        SimpleFeature f = fb.buildFeature(null);
        return f;
    }
}
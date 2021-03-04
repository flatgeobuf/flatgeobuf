package org.wololo.flatgeobuf.geotools;

import static java.nio.charset.CodingErrorAction.REPLACE;
import java.io.IOException;
import java.io.OutputStream;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.CharBuffer;
import java.nio.channels.Channels;
import java.nio.channels.WritableByteChannel;
import java.nio.charset.CharsetEncoder;
import java.nio.charset.StandardCharsets;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.LocalTime;
import java.time.OffsetDateTime;
import java.time.OffsetTime;
import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.opengis.feature.simple.SimpleFeature;
import org.wololo.flatgeobuf.generated.ColumnType;
import org.wololo.flatgeobuf.generated.Feature;
import org.wololo.flatgeobuf.generated.Geometry;
import com.google.flatbuffers.FlatBufferBuilder;

public class FeatureConversions {

    private static void writeString(ByteBuffer target, String value) {

        CharsetEncoder encoder = StandardCharsets.UTF_8.newEncoder().onMalformedInput(REPLACE)
                .onUnmappableCharacter(REPLACE);

        // save current position to write the string length later
        final int lengthPosition = target.position();
        // and leave room for it
        target.position(lengthPosition + Integer.BYTES);

        final int startStrPos = target.position();
        final boolean endOfInput = true;
        encoder.encode(CharBuffer.wrap(value), target, endOfInput);

        final int endStrPos = target.position();
        final int encodedLength = endStrPos - startStrPos;

        // absolute put, doesn't change the current position
        target.putInt(lengthPosition, encodedLength);
    }

    public static void serialize(SimpleFeature feature, HeaderMeta headerMeta,
            final OutputStream to) throws IOException {
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);
        org.locationtech.jts.geom.Geometry geometry =
                (org.locationtech.jts.geom.Geometry) feature.getDefaultGeometry();
        final int propertiesOffset = createProperiesVector(feature, builder, headerMeta);
        GeometryOffsets go =
                GeometryConversions.serialize(builder, geometry, headerMeta.geometryType);
        int geometryOffset = 0;
        if (go.gos != null && go.gos.length > 0) {
            int[] partOffsets = new int[go.gos.length];
            for (int i = 0; i < go.gos.length; i++) {
                GeometryOffsets goPart = go.gos[i];
                int partOffset = Geometry.createGeometry(builder, goPart.endsOffset,
                        goPart.coordsOffset, 0, 0, 0, 0, 0, 0);
                partOffsets[i] = partOffset;
            }
            int partsOffset = Geometry.createPartsVector(builder, partOffsets);
            geometryOffset = Geometry.createGeometry(builder, 0, 0, 0, 0, 0, 0, 0, partsOffset);
        } else {
            geometryOffset = Geometry.createGeometry(builder, go.endsOffset, go.coordsOffset, 0, 0,
                    0, 0, 0, 0);
        }
        int featureOffset = Feature.createFeature(builder, geometryOffset, propertiesOffset, 0);
        builder.finishSizePrefixed(featureOffset);

        WritableByteChannel channel = Channels.newChannel(to);
        ByteBuffer dataBuffer = builder.dataBuffer();
        while (dataBuffer.hasRemaining()) {
            channel.write(dataBuffer);
        }
    }

    /**
     * Writes the properties vector to {@code builder} and returns its offset
     */
    private static int createProperiesVector(SimpleFeature feature, FlatBufferBuilder builder,
            HeaderMeta headerMeta) {

        ByteBuffer propertiesVectorBuf = ByteBuffer.allocate(1024 * 1024);
        propertiesVectorBuf.order(ByteOrder.LITTLE_ENDIAN);
        for (short i = 0; i < headerMeta.columns.size(); i++) {
            ColumnMeta column = headerMeta.columns.get(i);
            byte type = column.type;
            Object value = feature.getAttribute(column.name);
            if (value == null)
                continue;
            propertiesVectorBuf.putShort(i);
            if (type == ColumnType.Bool)
                propertiesVectorBuf.put((byte) ((boolean) value ? 1 : 0));
            else if (type == ColumnType.Byte)
                propertiesVectorBuf.put((byte) value);
            else if (type == ColumnType.Short)
                propertiesVectorBuf.putShort((short) value);
            else if (type == ColumnType.Int)
                propertiesVectorBuf.putInt((int) value);
            else if (type == ColumnType.Long)
                if (value instanceof Long)
                    propertiesVectorBuf.putLong((long) value);
                else if (value instanceof BigInteger)
                    propertiesVectorBuf.putLong(((BigInteger) value).longValue());
                else
                    propertiesVectorBuf.putLong((long) value);
            else if (type == ColumnType.Double)
                if (value instanceof Double)
                    propertiesVectorBuf.putDouble((double) value);
                else if (value instanceof BigDecimal)
                    propertiesVectorBuf.putDouble(((BigDecimal) value).doubleValue());
                else
                    propertiesVectorBuf.putDouble((double) value);
            else if (type == ColumnType.DateTime) {
                String isoDateTime = "";
                if (value instanceof LocalDateTime)
                    isoDateTime = ((LocalDateTime) value).toString();
                else if (value instanceof LocalDate)
                    isoDateTime = ((LocalDate) value).toString();
                else if (value instanceof LocalTime)
                    isoDateTime = ((LocalTime) value).toString();
                else if (value instanceof OffsetDateTime)
                    isoDateTime = ((OffsetDateTime) value).toString();
                else if (value instanceof OffsetTime)
                    isoDateTime = ((OffsetTime) value).toString();
                else
                    throw new RuntimeException("Unknown date/time type " + type);
                writeString(propertiesVectorBuf, isoDateTime);
            } else if (type == ColumnType.String)
                writeString(propertiesVectorBuf, (String) value);
            else
                throw new RuntimeException("Unknown type " + type);
        }

        int propertiesOffset = 0;
        if (propertiesVectorBuf.position() > 0) {
            propertiesVectorBuf.flip();
            propertiesOffset = Feature.createPropertiesVector(builder, propertiesVectorBuf);
        }
        return propertiesOffset;
    }

    private static String readString(ByteBuffer bb, String name) {
        int length = bb.getInt();
        byte[] stringBytes = new byte[length];
        bb.get(stringBytes, 0, length);
        String value = new String(stringBytes, StandardCharsets.UTF_8);
        return value;
    }

    public static SimpleFeature deserialize(Feature feature, SimpleFeatureBuilder fb, HeaderMeta headerMeta,
            String fid) {
        Geometry geometry = feature.geometry();
        if (geometry == null)
            return null;
        org.locationtech.jts.geom.Geometry jtsGeometry = GeometryConversions.deserialize(geometry,
                headerMeta.geometryType);
        if (jtsGeometry != null)
            fb.add(GeometryConversions.deserialize(geometry, headerMeta.geometryType));
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
                else if (type == ColumnType.Byte)
                    fb.set(name, bb.get());
                else if (type == ColumnType.Short)
                    fb.set(name, bb.getShort());
                else if (type == ColumnType.Int)
                    fb.set(name, bb.getInt());
                else if (type == ColumnType.Long)
                    fb.set(name, bb.getLong());
                else if (type == ColumnType.Double)
                    fb.set(name, bb.getDouble());
                else if (type == ColumnType.DateTime)
                    fb.set(name, readString(bb, name));
                else if (type == ColumnType.String)
                    fb.set(name, readString(bb, name));
                else
                    throw new RuntimeException("Unknown type");
            }
        }
        SimpleFeature f = fb.buildFeature(fid);
        return f;
    }
}
package org.wololo.flatgeobuf;

import org.locationtech.jts.geom.Envelope;
import org.wololo.flatgeobuf.generated.Column;
import org.wololo.flatgeobuf.generated.Crs;
import org.wololo.flatgeobuf.generated.Header;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.channels.Channels;
import java.nio.channels.WritableByteChannel;
import java.util.ArrayList;
import java.util.List;

import com.google.common.io.LittleEndianDataInputStream;
import com.google.flatbuffers.ByteBufferUtil;
import com.google.flatbuffers.FlatBufferBuilder;

import static com.google.flatbuffers.Constants.SIZE_PREFIX_LENGTH;

public class HeaderMeta {
    public String name;
    public byte geometryType;
    public int srid;
    public Envelope envelope;
    public long featuresCount;
    public boolean hasZ = false;
    public boolean hasM = false;
    public boolean hasT = false;
    public boolean hasTM = false;
    public int indexNodeSize;
    public List<ColumnMeta> columns;
    public int offset;

    public static void write(HeaderMeta headerMeta, OutputStream to,
            FlatBufferBuilder builder) throws IOException {
        int[] columnsArray = headerMeta.columns.stream().mapToInt(c -> {
            int nameOffset = builder.createString(c.name);
            int type = c.type;
            return Column.createColumn(builder, nameOffset, type, 0, 0, c.width, c.precision, c.scale, c.nullable, c.unique,
                    c.primary_key, 0);
        }).toArray();
        int columnsOffset = Header.createColumnsVector(builder, columnsArray);

        int nameOffset = 0;
        if (headerMeta.name!=null) {
            nameOffset = builder.createString(headerMeta.name);
        }
        int crsOffset = 0;
        if (headerMeta.srid != 0) {
            Crs.startCrs(builder);
            Crs.addCode(builder, headerMeta.srid);
            crsOffset = Crs.endCrs(builder);
        }
        int envelopeOffset = 0;
        if (headerMeta.envelope != null) {
            envelopeOffset = Header.createEnvelopeVector(builder,
            new double[] { headerMeta.envelope.getMinX(), headerMeta.envelope.getMinY(), headerMeta.envelope.getMaxX(), headerMeta.envelope.getMaxY() });
        }
        Header.startHeader(builder);
        Header.addGeometryType(builder, headerMeta.geometryType);
        Header.addIndexNodeSize(builder, headerMeta.indexNodeSize );
        Header.addColumns(builder, columnsOffset);
        Header.addEnvelope(builder, envelopeOffset);
        Header.addName(builder, nameOffset);
        Header.addCrs(builder, crsOffset);
        Header.addFeaturesCount(builder, headerMeta.featuresCount);
        int offset = Header.endHeader(builder);

        builder.finishSizePrefixed(offset);

        WritableByteChannel channel = Channels.newChannel(to);
        ByteBuffer dataBuffer = builder.dataBuffer();
        while (dataBuffer.hasRemaining())
            channel.write(dataBuffer);
    }

    public static HeaderMeta read(ByteBuffer bb) throws IOException {
        int offset = 0;
        if (!Constants.isFlatgeobuf(bb))
            throw new IOException("This is not a flatgeobuf!");
        bb.position(offset += Constants.MAGIC_BYTES.length);
        int headerSize = ByteBufferUtil.getSizePrefix(bb);
        bb.position(offset += SIZE_PREFIX_LENGTH);
        Header header = Header.getRootAsHeader(bb);
        bb.position(offset += headerSize);
        int geometryType = header.geometryType();

        HeaderMeta headerMeta = new HeaderMeta();

        headerMeta.featuresCount = header.featuresCount();
        headerMeta.indexNodeSize = header.indexNodeSize();

        int columnsLength = header.columnsLength();
        ArrayList<ColumnMeta> columnMetas = new ArrayList<ColumnMeta>();
        for (int i = 0; i < columnsLength; i++) {
            ColumnMeta columnMeta = new ColumnMeta();
            columnMeta.name = header.columns(i).name();
            columnMeta.type = (byte) header.columns(i).type();
            columnMeta.title = header.columns(i).title();
            columnMeta.description = header.columns(i).description();
            columnMeta.width = header.columns(i).width();
            columnMeta.precision = header.columns(i).precision();
            columnMeta.scale = header.columns(i).scale();
            columnMeta.nullable = header.columns(i).nullable();
            columnMeta.unique = header.columns(i).unique();
            columnMeta.nullable = header.columns(i).nullable();
            columnMeta.primary_key = header.columns(i).primaryKey();
            columnMeta.metadata = header.columns(i).metadata();
            columnMetas.add(columnMeta);
        }

        Crs crs = header.crs();
        if (crs != null && crs.code() != 0)
            headerMeta.srid = crs.code();
        if (header.envelopeLength() == 4) {
            double minX = header.envelope(0);
            double minY = header.envelope(1);
            double maxX = header.envelope(2);
            double maxY = header.envelope(3);
            headerMeta.envelope = new Envelope(minX, maxX, minY, maxY);
        }

        headerMeta.columns = columnMetas;
        headerMeta.geometryType = (byte) geometryType;
        headerMeta.offset = offset;

        return headerMeta;
    }

    public static HeaderMeta read(InputStream stream) throws IOException {
        LittleEndianDataInputStream data = new LittleEndianDataInputStream(stream);
        byte[] magicbytes = new byte[8];
        int len;
        data.readFully(magicbytes);
        len = data.readInt();
        byte[] bytes = new byte[len];
        data.readFully(bytes);
        ByteBuffer bb = ByteBuffer.allocateDirect(8 + 4 + len);
        bb.mark();
        bb.put(magicbytes);
        bb.putInt(len);
        bb.put(bytes);
        bb.reset();
        return HeaderMeta.read(bb);
    }
}
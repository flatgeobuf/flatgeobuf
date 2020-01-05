package org.wololo.flatgeobuf.geotools;

import com.google.flatbuffers.ByteBufferUtil;
import com.google.flatbuffers.FlatBufferBuilder;
import org.geotools.feature.simple.SimpleFeatureTypeBuilder;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.geom.MultiPoint;
import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;
import org.opengis.feature.type.AttributeDescriptor;
import org.opengis.feature.type.GeometryDescriptor;
import org.wololo.flatgeobuf.Constants;
import org.wololo.flatgeobuf.generated.Column;
import org.wololo.flatgeobuf.generated.ColumnType;
import org.wololo.flatgeobuf.generated.GeometryType;
import org.wololo.flatgeobuf.generated.Header;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.List;

import static com.google.flatbuffers.Constants.SIZE_PREFIX_LENGTH;

public class FeatureTypeConversions {

    public static HeaderMeta serialize(SimpleFeatureType featureType, SimpleFeature feature, long featuresCount,
            OutputStream outputStream) throws IOException {

        List<AttributeDescriptor> types = featureType.getAttributeDescriptors();
        List<ColumnMeta> columns = new ArrayList<ColumnMeta>();

        for (int i = 0; i < types.size(); i++) {
            AttributeDescriptor ad = types.get(i);
            if (ad instanceof GeometryDescriptor) {
                // multiple geometries per feature is not supported
            } else {
                String key = ad.getLocalName();
                Class<?> binding = ad.getType().getBinding();
                ColumnMeta column = new ColumnMeta();
                column.name = key;
                if (binding.isAssignableFrom(Boolean.class))
                    column.type = ColumnType.Bool;
                else if (binding.isAssignableFrom(Integer.class))
                    column.type = ColumnType.Int;
                else if (binding.isAssignableFrom(Long.class))
                    column.type = ColumnType.Long;
                else if (binding.isAssignableFrom(Double.class))
                    column.type = ColumnType.Double;
                else if (binding.isAssignableFrom(String.class))
                    column.type = ColumnType.String;
                else
                    throw new RuntimeException("Unknown type");
                columns.add(column);
            }
        }

        // CoordinateReferenceSystem crs = featureType.getGeometryDescriptor().getCoordinateReferenceSystem();
        byte geometryType = GeometryConversions
                .toGeometryType(featureType.getGeometryDescriptor().getType().getBinding());
        // byte dimensions = (byte) (crs == null ? 2 : crs.getCoordinateSystem().getDimension());

        outputStream.write(Constants.MAGIC_BYTES);

        HeaderMeta headerMeta = new HeaderMeta();
        headerMeta.featuresCount = featuresCount;
        headerMeta.geometryType = geometryType;
        headerMeta.columns = columns;

        org.locationtech.jts.geom.Geometry geometry = (org.locationtech.jts.geom.Geometry) feature.getDefaultGeometry();
        if (!geometry.isEmpty()) {
            Coordinate coord = geometry.getCoordinate();
            if (!Double.isNaN(coord.getZ()))
                headerMeta.hasZ = true;
            if (!Double.isNaN(coord.getM()))
                headerMeta.hasM = true;
        }

        byte[] headerBuffer = buildHeader(headerMeta);
        outputStream.write(headerBuffer);

        return headerMeta;
    }

    private static byte[] buildHeader(HeaderMeta headerMeta) {
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);

        int[] columnsArray = headerMeta.columns.stream().mapToInt(c -> {
            int nameOffset = builder.createString(c.name);
            int type = c.type;
            return Column.createColumn(builder, nameOffset, type);
        }).toArray();
        int columnsOffset = Header.createColumnsVector(builder, columnsArray);

        Header.startHeader(builder);
        Header.addGeometryType(builder, headerMeta.geometryType);
        Header.addHasZ(builder, headerMeta.hasZ);
        Header.addHasM(builder, headerMeta.hasM);
        Header.addIndexNodeSize(builder, 0);
        Header.addColumns(builder, columnsOffset);
        Header.addFeaturesCount(builder, headerMeta.featuresCount);
        int offset = Header.endHeader(builder);

        builder.finishSizePrefixed(offset);

        return builder.sizedByteArray();
    }

    public static HeaderMeta deserialize(ByteBuffer bb, String name, String geometryPropertyName) throws IOException {
        int offset = 0;
        if (Constants.isFlatgeobuf(bb)) {
            throw new IOException("This is not a flatgeobuf!");
        }
        bb.position(offset += Constants.MAGIC_BYTES.length);
        int headerSize = ByteBufferUtil.getSizePrefix(bb);
        bb.position(offset += SIZE_PREFIX_LENGTH);
        Header header = Header.getRootAsHeader(bb);
        bb.position(offset += headerSize);
        int geometryType = header.geometryType();
        Class<?> geometryClass;
        switch (geometryType) {
            case GeometryType.Point:
                geometryClass = Point.class;
                break;
            case GeometryType.MultiPoint:
                geometryClass = MultiPoint.class;
                break;
            case GeometryType.LineString:
                geometryClass = LineString.class;
                break;
            case GeometryType.MultiLineString:
                geometryClass = MultiLineString.class;
                break;
            case GeometryType.Polygon:
                geometryClass = Polygon.class;
                break;
            case GeometryType.MultiPolygon:
                geometryClass = MultiPolygon.class;
                break;
            default:
                throw new RuntimeException("Unknown geometry type");
        }

        int columnsLength = header.columnsLength();
        ArrayList<ColumnMeta> columnMetas = new ArrayList<ColumnMeta>();
        for (int i = 0; i < columnsLength; i++) {
            ColumnMeta columnMeta = new ColumnMeta();
            columnMeta.name = header.columns(i).name();
            columnMeta.type = (byte) header.columns(i).type();
            columnMetas.add(columnMeta);
        }

        SimpleFeatureTypeBuilder ftb = new SimpleFeatureTypeBuilder();
        ftb.setName(name);
        ftb.add(geometryPropertyName, geometryClass);
        for (ColumnMeta columnMeta : columnMetas) {
            ftb.add(columnMeta.name, columnMeta.getBinding());
        }
        SimpleFeatureType ft = ftb.buildFeatureType();

        HeaderMeta headerMeta = new HeaderMeta();
        headerMeta.columns = columnMetas;
        headerMeta.geometryType = (byte) geometryType;
        headerMeta.hasZ = header.hasZ();
        headerMeta.hasT = header.hasT();
        headerMeta.offset = offset;
        headerMeta.featureType = ft;

        return headerMeta;
    }

}

package org.wololo.flatgeobuf;

import java.io.IOException;
import java.util.Arrays;
import java.util.function.IntFunction;
import java.util.function.Supplier;
import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.CoordinateSequence;
import org.locationtech.jts.geom.CoordinateSequenceFilter;
import org.locationtech.jts.geom.GeometryCollection;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.LinearRing;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.MultiPoint;
import org.locationtech.jts.geom.MultiPolygon;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.geom.Polygon;
import org.wololo.flatgeobuf.generated.Geometry;
import org.wololo.flatgeobuf.generated.GeometryType;
import com.google.flatbuffers.FlatBufferBuilder;

public class GeometryConversions {
    public static GeometryOffsets serializePart(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry,
            int geometryType) throws IOException {
        GeometryOffsets go = new GeometryOffsets();

        if (geometry == null)
            return go;

        if (geometryType == GeometryType.MultiLineString) {
            int end = 0;
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                go.ends = new long[mls.getNumGeometries()];
                for (int i = 0; i < mls.getNumGeometries(); i++)
                    go.ends[i] = end += mls.getGeometryN(i).getNumPoints();
            }
        } else if (geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            go.ends = new long[p.getNumInteriorRing() + 1];
            int end = p.getExteriorRing().getNumPoints();
            go.ends[0] = end;
            for (int i = 0; i < p.getNumInteriorRing(); i++)
                go.ends[i + 1] = end += p.getInteriorRingN(i).getNumPoints();
        } else if (geometryType == GeometryType.MultiPolygon) {
            MultiPolygon mp = (MultiPolygon) geometry;
            int numGeometries = mp.getNumGeometries();
            GeometryOffsets[] gos = new GeometryOffsets[numGeometries];
            for (int i = 0; i < numGeometries; i++) {
                Polygon p = (Polygon) mp.getGeometryN(i);
                gos[i] = serializePart(builder, p, GeometryType.Polygon);
            }
            go.gos = gos;
            return go;
        }

        final int numPoints = geometry.getNumPoints();
        // build the vector "manually", using a CoordinateSequenceFilter to avoid creating
        // Coordinate arrays or any Coordinate at all, depending on the underlying
        // CoordinateSequence implementation. Vector elements ought to be added in reverse order
        Geometry.startXyVector(builder, 2 * numPoints);
        applyInReverseOrder(geometry, new ReverseXYCoordinateSequenceFilter(builder));
        go.coordsOffset = builder.endVector();

        if (go.ends != null)
            go.endsOffset = Geometry.createEndsVector(builder, go.ends);

        go.type = geometryType;

        return go;
    }

    public static int serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry, byte geometryType) throws IOException {
        byte knownGeometryType = geometryType;
        if (geometryType == GeometryType.Unknown)
            knownGeometryType = GeometryConversions.toGeometryType(geometry.getClass());

        GeometryOffsets go =
                GeometryConversions.serializePart(builder, geometry, knownGeometryType);

        int geometryOffset;
        if (go.gos != null && go.gos.length > 0) {
            int[] partOffsets = new int[go.gos.length];
            for (int i = 0; i < go.gos.length; i++) {
                GeometryOffsets goPart = go.gos[i];
                int partOffset = Geometry.createGeometry(builder, goPart.endsOffset,
                        goPart.coordsOffset, 0, 0, 0, 0, goPart.type, 0);
                partOffsets[i] = partOffset;
            }
            int partsOffset = Geometry.createPartsVector(builder, partOffsets);
            geometryOffset = Geometry.createGeometry(builder, 0, 0, 0, 0, 0, 0, geometryType == GeometryType.Unknown ? knownGeometryType : 0, partsOffset);
        } else {
            geometryOffset = Geometry.createGeometry(builder, go.endsOffset, go.coordsOffset, 0, 0,
                    0, 0, geometryType == GeometryType.Unknown ? knownGeometryType : 0, 0);
        }
        return geometryOffset;
    }

    /**
     * Applies the {@code filter} to all {@link org.locationtech.jts.geom.Geometry#getGeometryN(int)
     * subgeometries} in reverse order if it's a {@link GeometryCollection} or a {@link Polygon}
     * (i.e. interior rings in reverse order first)
     */
    private static void applyInReverseOrder(org.locationtech.jts.geom.Geometry geometry,
            CoordinateSequenceFilter filter) {

        final int numGeometries = geometry.getNumGeometries();
        if (numGeometries > 1) {
            for (int i = numGeometries - 1; i >= 0; i--) {
                org.locationtech.jts.geom.Geometry sub = geometry.getGeometryN(i);
                applyInReverseOrder(sub, filter);
            }
        } else if (geometry instanceof Polygon) {
            Polygon p = (Polygon) geometry;
            for (int i = p.getNumInteriorRing() - 1; i >= 0; i--) {
                org.locationtech.jts.geom.Geometry hole = p.getInteriorRingN(i);
                applyInReverseOrder(hole, filter);
            }
            applyInReverseOrder(p.getExteriorRing(), filter);
        } else {
            geometry.apply(filter);
        }
    }

    private static class ReverseXYCoordinateSequenceFilter implements CoordinateSequenceFilter {
        private FlatBufferBuilder builder;

        ReverseXYCoordinateSequenceFilter(FlatBufferBuilder builder) {
            this.builder = builder;
        }

        public @Override void filter(final CoordinateSequence seq, final int coordIndex) {
            int reverseSeqIndex = seq.size() - coordIndex - 1;
            double y = seq.getOrdinate(reverseSeqIndex, 1);
            double x = seq.getOrdinate(reverseSeqIndex, 0);
            builder.addDouble(y);
            builder.addDouble(x);
        }

        public @Override boolean isGeometryChanged() {
            return false;
        }

        public @Override boolean isDone() {
            return false;
        }
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Geometry geometry, int geometryType) {
        GeometryFactory factory = new GeometryFactory();

        switch (geometryType) {
        case GeometryType.MultiPolygon:
            int partsLength = geometry.partsLength();
            Polygon[] polygons = new Polygon[partsLength];
            for (int i = 0; i < geometry.partsLength(); i++) {
                polygons[i] = (Polygon) deserialize(geometry.parts(i), GeometryType.Polygon);
            }
            return factory.createMultiPolygon(polygons);
        }

        int xyLength = geometry.xyLength();
        Coordinate[] coordinates = new Coordinate[xyLength >> 1];
        int c = 0;
        for (int i = 0; i < xyLength; i = i + 2)
            coordinates[c++] = new Coordinate(geometry.xy(i), geometry.xy(i + 1));

        IntFunction<Polygon> makePolygonWithRings = (int endsLength) -> {
            LinearRing[] lrs = new LinearRing[endsLength];
            int s = 0;
            for (int i = 0; i < endsLength; i++) {
                int e = (int) geometry.ends(i);
                Coordinate[] cs = Arrays.copyOfRange(coordinates, s, e);
                lrs[i] = factory.createLinearRing(cs);
                s = e;
            }
            LinearRing shell = lrs[0];
            LinearRing holes[] = Arrays.copyOfRange(lrs, 1, endsLength);
            return factory.createPolygon(shell, holes);
        };

        Supplier<Polygon> makePolygon = () -> {
            int endsLength = geometry.endsLength();
            if (endsLength > 1)
                return makePolygonWithRings.apply(endsLength);
            else
                return factory.createPolygon(coordinates);
        };

        switch (geometryType) {
        case GeometryType.Unknown:
            return null;
        case GeometryType.Point:
            if (coordinates.length > 0) {
                return factory.createPoint(coordinates[0]);
            } else {
                return factory.createPoint();
            }
        case GeometryType.MultiPoint:
            return factory.createMultiPointFromCoords(coordinates);
        case GeometryType.LineString:
            return factory.createLineString(coordinates);
        case GeometryType.MultiLineString: {
            int lengthLengths = geometry.endsLength();
            if (lengthLengths < 2)
                return factory.createMultiLineString(new LineString[] { factory.createLineString(coordinates) });
            LineString[] lss = new LineString[lengthLengths];
            int s = 0;
            for (int i = 0; i < lengthLengths; i++) {
                int e = (int) geometry.ends(i);
                Coordinate[] cs = Arrays.copyOfRange(coordinates, s, e);
                lss[i] = factory.createLineString(cs);
                s = e;
            }
            return factory.createMultiLineString(lss);
        }
        case GeometryType.Polygon:
            return makePolygon.get();
        default:
            throw new RuntimeException("Unknown geometry type");
        }
    }

    public static Class<?> getGeometryClass(int geometryType) {
        switch (geometryType) {
        case GeometryType.Unknown:
            return Geometry.class;
        case GeometryType.Point:
            return Point.class;
        case GeometryType.MultiPoint:
            return MultiPoint.class;
        case GeometryType.LineString:
            return LineString.class;
        case GeometryType.MultiLineString:
            return MultiLineString.class;
        case GeometryType.Polygon:
            return Polygon.class;
        case GeometryType.MultiPolygon:
            return MultiPolygon.class;
        default:
            throw new RuntimeException("Unknown geometry type");
        }
    }

    public static byte toGeometryType(Class<?> geometryClass) {

        if (geometryClass == org.locationtech.jts.geom.Geometry.class)
            return GeometryType.Unknown;
        else if (MultiPoint.class.isAssignableFrom(geometryClass))
            return GeometryType.MultiPoint;
        else if (Point.class.isAssignableFrom(geometryClass))
            return GeometryType.Point;
        else if (MultiLineString.class.isAssignableFrom(geometryClass))
            return GeometryType.MultiLineString;
        else if (LineString.class.isAssignableFrom(geometryClass))
            return GeometryType.LineString;
        else if (MultiPolygon.class.isAssignableFrom(geometryClass))
            return GeometryType.MultiPolygon;
        else if (Polygon.class.isAssignableFrom(geometryClass))
            return GeometryType.Polygon;
        else
            throw new RuntimeException("Unknown geometry type");
    }
}
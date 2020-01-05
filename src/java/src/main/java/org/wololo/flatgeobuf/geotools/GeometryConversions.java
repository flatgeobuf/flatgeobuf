package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.nio.DoubleBuffer;
import java.util.stream.Stream;
import java.util.Arrays;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Point;
import org.locationtech.jts.geom.MultiPoint;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.LinearRing;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.MultiPolygon;
import org.locationtech.jts.geom.CoordinateSequence;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.impl.PackedCoordinateSequence;

import org.wololo.flatgeobuf.generated.*;
import org.wololo.flatgeobuf.generated.Geometry;

public class GeometryConversions {
    public static GeometryOffsets serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry,
            byte geometryType, HeaderMeta headerMeta) throws IOException {
        GeometryOffsets go = new GeometryOffsets();

        if (geometryType == GeometryType.MultiLineString) {
            int end = 0;
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                go.ends = new int[mls.getNumGeometries()];
                for (int i = 0; i < mls.getNumGeometries(); i++)
                    go.ends[i] = end += mls.getGeometryN(i).getNumPoints();
            }
        } else if (geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            go.ends = new int[p.getNumInteriorRing() + 1];
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
                gos[i] = serialize(builder, p, GeometryType.Polygon, headerMeta);
            }
            go.gos = gos;
            return go;
        }

        go.xyOffset = Geometry.createXyVector(builder, Stream.of(geometry.getCoordinates()).flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray());
        if (headerMeta.hasZ)
            go.zOffset = Geometry.createZVector(builder, Stream.of(geometry.getCoordinates()).mapToDouble(c -> c.getZ()).toArray());
        if (headerMeta.hasM)
            go.mOffset = Geometry.createMVector(builder, Stream.of(geometry.getCoordinates()).mapToDouble(c -> c.getM()).toArray());

        if (go.ends != null)
            go.endsOffset = Geometry.createEndsVector(builder, go.ends);

        return go;
    }

    private static Polygon makePolygonWithRings(GeometryFactory factory, Geometry geometry, double[] cs, int size, int dim, int measures, int endsLength) {
        LinearRing[] lrs = new LinearRing[endsLength];
        int s = 0;
        for (int i = 0; i < endsLength; i++) {
            int e = (int) geometry.ends(i);
            CoordinateSequence partialCoordinateSequence = new PackedCoordinateSequence.Double(
                    Arrays.copyOfRange(cs, s * size, e * size),
                    dim, measures);
            lrs[i] = factory.createLinearRing(partialCoordinateSequence);
            s = e;
        }
        LinearRing shell = lrs[0];
        LinearRing holes[] = Arrays.copyOfRange(lrs, 1, endsLength);
        return factory.createPolygon(shell, holes);
    }

    private static Polygon makePolygon(GeometryFactory factory, Geometry geometry, CoordinateSequence coordinateSequence, double[] cs, int size, int dim, int measures) {
        int endsLength = geometry.endsLength();
        if (endsLength > 1)
            return makePolygonWithRings(factory, geometry, cs, size, dim, measures, endsLength);
        else
            return factory.createPolygon(coordinateSequence);
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Geometry geometry, byte geometryType, HeaderMeta headerMeta) {
        GeometryFactory factory = new GeometryFactory();
        
        switch (geometryType) {
        case GeometryType.MultiPolygon:
            int partsLength = geometry.partsLength();
            Polygon[] polygons = new Polygon[partsLength];
            for (int i = 0; i < geometry.partsLength(); i++)
                polygons[i] = (Polygon) deserialize(geometry.parts(i), GeometryType.Polygon, headerMeta);
            return factory.createMultiPolygon(polygons);
        }
        
        int dim = 2;
        if (headerMeta.hasZ)
            dim++;
        int measures = 0;
        if (headerMeta.hasM)
            measures++;
        int coordinateSize = dim + measures; // number of doubles per coordinate
        double[] coordinateDoubleArray = null;
        if (dim > 2) {
            // TODO: does not handle measure only
            // NOTE: inefficient, need PackedCoordinateSequence variant with similar memory model
            coordinateDoubleArray = new double[(geometry.xyLength() / 2) * coordinateSize];
            for (int i = 0; i < coordinateDoubleArray.length; i += coordinateSize) {
                coordinateDoubleArray[i] = geometry.xy((i / coordinateSize) * 2);
                coordinateDoubleArray[i + 1] = geometry.xy((i / coordinateSize) * 2 + 1);
                if (headerMeta.hasZ)
                    coordinateDoubleArray[i + 2] = geometry.z((i / coordinateSize));
                if (headerMeta.hasM)
                    coordinateDoubleArray[i + 3] = geometry.m((i / coordinateSize));
            }
        } else {
            DoubleBuffer coordinateDoubleBuffer = geometry.xyAsByteBuffer().asDoubleBuffer();
            coordinateDoubleArray = new double[coordinateDoubleBuffer.remaining()];
            coordinateDoubleBuffer.get(coordinateDoubleArray);
        }
        CoordinateSequence coordinateSequence = new PackedCoordinateSequence.Double(coordinateDoubleArray, dim, measures);

        switch (geometryType) {
        case GeometryType.Point:
            if (coordinateSequence.size() > 0)
                return factory.createPoint(coordinateSequence.getCoordinate(0));
            else
                return factory.createPoint();
        case GeometryType.MultiPoint:
            return factory.createMultiPoint(coordinateSequence);
        case GeometryType.LineString:
            return factory.createLineString(coordinateSequence);
        case GeometryType.MultiLineString: {
            int lengthLengths = geometry.endsLength();
            if (lengthLengths < 2)
                return factory.createMultiLineString(new LineString[] { factory.createLineString(coordinateSequence) });
            LineString[] lss = new LineString[lengthLengths];
            int s = 0;
            for (int i = 0; i < lengthLengths; i++) {
                int e = (int) geometry.ends(i);
                CoordinateSequence partialCoordinateSequence = new PackedCoordinateSequence.Double(
                        Arrays.copyOfRange(coordinateDoubleArray, s * coordinateSize, e * coordinateSize),
                        dim, measures);
                lss[i] = factory.createLineString(partialCoordinateSequence);
                s = e;
            }
            return factory.createMultiLineString(lss);
        }
        case GeometryType.Polygon:
            return makePolygon(factory, geometry, coordinateSequence, coordinateDoubleArray, coordinateSize, dim, measures);
        default:
            throw new RuntimeException("Unknown geometry type");
        }
    }

    public static byte toGeometryType(Class<?> geometryClass) {
        if (geometryClass.isAssignableFrom(MultiPoint.class))
            return GeometryType.MultiPoint;
        else if (geometryClass.isAssignableFrom(Point.class))
            return GeometryType.Point;
        else if (geometryClass.isAssignableFrom(MultiLineString.class))
            return GeometryType.MultiLineString;
        else if (geometryClass.isAssignableFrom(LineString.class))
            return GeometryType.LineString;
        else if (geometryClass.isAssignableFrom(MultiPolygon.class))
            return GeometryType.MultiPolygon;
        else if (geometryClass.isAssignableFrom(Polygon.class))
            return GeometryType.Polygon;
        else
            throw new RuntimeException("Unknown geometry type");
    }
}
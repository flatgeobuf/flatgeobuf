package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.DoubleBuffer;
import java.util.stream.Stream;
import java.util.Arrays;
import java.util.function.IntFunction;
import java.util.function.Supplier;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.CoordinateSequence;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.LinearRing;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;

import org.locationtech.jts.geom.impl.PackedCoordinateSequence;
import org.wololo.flatgeobuf.generated.*;

public class GeometryConversions {
    public static GeometryOffsets serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry,
    		byte geometryType) throws IOException {
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
                gos[i] = serialize(builder, p, GeometryType.Polygon);
            }
        	go.gos = gos;
        	return go;
        }
        
        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        //if (headerMeta.hasZ && headerMeta.hasM)
        //    coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        //else if (headerMeta.hasZ || headerMeta.hasM)
        //    coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        //else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        go.coordsOffset = Geometry.createXyVector(builder, coords);
        
        if (go.ends != null)
            go.endsOffset = Geometry.createEndsVector(builder, go.ends);

        return go;
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Geometry geometry, byte geometryType) {
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

        int dim = 2; // check for z/t and increase accordingly
        int measures = 0; // check for m and increase accordingly
        int coordinateSize = dim + measures; // number of doubles per coordinate
        DoubleBuffer coordinateDoubleBuffer = geometry.xyAsByteBuffer().asDoubleBuffer();
        double[] coordinateDoubleArray = new double[coordinateDoubleBuffer.remaining()];
        coordinateDoubleBuffer.get(coordinateDoubleArray);
        CoordinateSequence coordinateSequence = new PackedCoordinateSequence.Double(coordinateDoubleArray, dim, measures);

        IntFunction<Polygon> makePolygonWithRings = (int endsLength) -> {
            LinearRing[] lrs = new LinearRing[endsLength];
            int s = 0;
            for (int i = 0; i < endsLength; i++) {
                int e = (int) geometry.ends(i);
                CoordinateSequence partialCoordinateSequence = new PackedCoordinateSequence.Double(
                        Arrays.copyOfRange(coordinateDoubleArray, s * coordinateSize, e * coordinateSize),
                        dim, measures);
                lrs[i] = factory.createLinearRing(partialCoordinateSequence);
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
                return factory.createPolygon(coordinateSequence);
        };

        switch (geometryType) {
        case GeometryType.Point:
            if (coordinateSequence.size() > 0) {
                return factory.createPoint(coordinateSequence.getCoordinate(0));
            } else {
                return factory.createPoint();
            }
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
            return makePolygon.get();
        default:
            throw new RuntimeException("Unknown geometry type");
        }
    }
}
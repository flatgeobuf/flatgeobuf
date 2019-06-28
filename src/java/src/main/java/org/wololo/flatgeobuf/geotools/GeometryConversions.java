package org.wololo.flatgeobuf.geotools;

import java.io.IOException;
import java.util.stream.Stream;
import java.util.Arrays;
import java.util.function.IntFunction;
import java.util.function.Supplier;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.LinearRing;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;

import org.wololo.flatgeobuf.generated.*;

public class GeometryConversions {
    public static GeometryOffsets serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry,
            int geometryType, int dimensions) throws IOException {
        GeometryOffsets go = new GeometryOffsets();

        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        if (dimensions == 4)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        else if (dimensions == 3)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        go.coordsOffset = Feature.createCoordsVector(builder, coords);

        if (geometryType == GeometryType.MultiLineString) {
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                go.ends = new int[mls.getNumGeometries()];
                for (int i = 0; i < mls.getNumGeometries(); i++)
                    go.ends[i] = mls.getGeometryN(i).getNumPoints() * dimensions;
            }
        } else if (geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            go.ends = new int[p.getNumInteriorRing() + 1];
            go.ends[0] = p.getExteriorRing().getNumPoints() * dimensions;
            for (int i = 0; i < p.getNumInteriorRing(); i++)
                go.ends[i + 1] = p.getInteriorRingN(i).getNumPoints() * dimensions;
        } else if (geometryType == GeometryType.MultiPolygon) {
            MultiPolygon mp = (MultiPolygon) geometry;
            if (mp.getNumGeometries() == 1) {
                Polygon p = (Polygon) mp.getGeometryN(0);
                go.ends = new int[p.getNumInteriorRing() + 1];
                go.ends[0] = p.getExteriorRing().getNumPoints() * dimensions;
                for (int i = 0; i < p.getNumInteriorRing(); i++)
                    go.ends[i + 1] = p.getInteriorRingN(i).getNumPoints() * dimensions;
            } else {
                go.ends = new int[mp.getNumGeometries()];
                go.endss = new int[mp.getNumGeometries()];
                int c = 0;
                for (int j = 0; j < mp.getNumGeometries(); j++) {
                    Polygon p = (Polygon) mp.getGeometryN(j);
                    c++;
                    for (int i = 0; i < p.getNumInteriorRing(); i++)
                        c++;
                }
                go.ends = new int[c];
                c = 0;
                for (int j = 0; j < mp.getNumGeometries(); j++) {
                    Polygon p = (Polygon) mp.getGeometryN(j);
                    int ringCount = 0;
                    int ringLength = p.getExteriorRing().getNumPoints() * dimensions;
                    go.ends[c++] = ringLength;
                    ringCount++;
                    for (int i = 0; i < p.getNumInteriorRing(); i++) {
                        ringLength = p.getInteriorRingN(i).getNumPoints() * dimensions;
                        go.ends[c++] = ringLength;
                        ringCount++;
                    }
                    go.endss[j] = ringCount;
                }
            }
        }
        if (go.ends != null)
            go.endsOffset = Feature.createEndsVector(builder, go.ends);
        if (go.endss != null)
            go.endssOffset = Feature.createEndssVector(builder, go.endss);

        return go;
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Feature feature, int geometryType, int dimensions) {
        GeometryFactory factory = new GeometryFactory();
        int coordsLength = feature.coordsLength();
        int dimLengths = coordsLength / dimensions;
        Coordinate[] coordinates = new Coordinate[dimLengths];
        int c = 0;
        for (int i = 0; i < coordsLength; i = i + dimensions)
            coordinates[c++] = new Coordinate(feature.coords(i), feature.coords(i + 1));

        IntFunction<Polygon> makePolygonWithRings = (int endsLength) -> {
            LinearRing[] lrs = new LinearRing[endsLength];
            int offset = 0;
            for (int i = 0; i < endsLength; i++) {
                int ringLength = (int) feature.ends(i) / dimensions;
                Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + ringLength);
                lrs[i] = factory.createLinearRing(cs);
                offset += ringLength;
            }
            LinearRing shell = lrs[0];
            LinearRing holes[] = Arrays.copyOfRange(lrs, 1, endsLength);
            return factory.createPolygon(shell, holes);
        };

        Supplier<Polygon> makePolygon = () -> {
            int endsLength = feature.endsLength();
            if (endsLength > 1) {
                return makePolygonWithRings.apply(endsLength);
            } else {
                return factory.createPolygon(coordinates);
            }
        };

        switch (geometryType) {
        case GeometryType.Point:
            return factory.createPoint(coordinates[0]);
        case GeometryType.MultiPoint:
            return factory.createMultiPointFromCoords(coordinates);
        case GeometryType.LineString:
            return factory.createLineString(coordinates);
        case GeometryType.MultiLineString: {
            int lengthLengths = feature.endsLength();
            LineString[] lss = new LineString[lengthLengths];
            int offset = 0;
            for (int i = 0; i < lengthLengths; i++) {
                int length = (int) feature.ends(i) / dimensions;
                Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + length);
                lss[i] = factory.createLineString(cs);
                offset += length;
            }
            return factory.createMultiLineString(lss);
        }
        case GeometryType.Polygon:
            return makePolygon.get();
        case GeometryType.MultiPolygon: {
            int endssLength = feature.endssLength();
            if (endssLength > 1) {
                Polygon[] ps = new Polygon[endssLength];
                int offset = 0;
                int roffset = 0;
                for (int j = 0; j < endssLength; j++) {
                    int ringCount = (int) feature.endss(j);
                    LinearRing[] lrs = new LinearRing[ringCount];
                    for (int i = 0; i < ringCount; i++) {
                        int ringLength = (int) feature.ends(roffset + i) / dimensions;
                        Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + ringLength);
                        lrs[i] = factory.createLinearRing(cs);
                        offset += ringLength;
                    }
                    roffset += ringCount;
                    LinearRing shell = lrs[0];
                    LinearRing holes[] = Arrays.copyOfRange(lrs, 1, ringCount);
                    ps[j] = factory.createPolygon(shell, holes);
                }
                return factory.createMultiPolygon(ps);
            } else {
                Polygon polygon = makePolygon.get();
                return factory.createMultiPolygon(new Polygon[] { polygon });
            }
        }
        default:
            throw new RuntimeException("Unknown geometry type");
        }
    }
}
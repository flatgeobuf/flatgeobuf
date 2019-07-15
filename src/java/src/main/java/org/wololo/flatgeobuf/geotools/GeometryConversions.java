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
            HeaderMeta headerMeta) throws IOException {
        GeometryOffsets go = new GeometryOffsets();

        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        //if (headerMeta.hasZ && headerMeta.hasM)
        //    coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        //else if (headerMeta.hasZ || headerMeta.hasM)
        //    coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        //else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        go.coordsOffset = Feature.createXyVector(builder, coords);

        if (headerMeta.geometryType == GeometryType.MultiLineString) {
            int end = 0;
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                go.ends = new int[mls.getNumGeometries()];
                for (int i = 0; i < mls.getNumGeometries(); i++)
                    go.ends[i] = end += mls.getGeometryN(i).getNumPoints();
            }
        } else if (headerMeta.geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            go.ends = new int[p.getNumInteriorRing() + 1];
            int end = p.getExteriorRing().getNumPoints();
            go.ends[0] = end;
            for (int i = 0; i < p.getNumInteriorRing(); i++)
                go.ends[i + 1] = end += p.getInteriorRingN(i).getNumPoints();
        } else if (headerMeta.geometryType == GeometryType.MultiPolygon) {
            int end = 0;
            MultiPolygon mp = (MultiPolygon) geometry;
            if (mp.getNumGeometries() == 1) {
                Polygon p = (Polygon) mp.getGeometryN(0);
                go.ends = new int[p.getNumInteriorRing() + 1];
                end = p.getExteriorRing().getNumPoints();
                go.ends[0] = end;
                for (int i = 0; i < p.getNumInteriorRing(); i++)
                    go.ends[i + 1] = end += p.getInteriorRingN(i).getNumPoints();
            } else {
                go.lengths = new int[mp.getNumGeometries()];
                int c = 0;
                for (int j = 0; j < mp.getNumGeometries(); j++) {
                    Polygon p = (Polygon) mp.getGeometryN(j);
                    c += p.getNumInteriorRing() + 1;
                }
                go.ends = new int[c];
                c = 0;
                for (int j = 0; j < mp.getNumGeometries(); j++) {
                    Polygon p = (Polygon) mp.getGeometryN(j);
                    go.ends[c++] = end += p.getExteriorRing().getNumPoints();
                    for (int i = 0; i < p.getNumInteriorRing(); i++)
                        go.ends[c++] = end += p.getInteriorRingN(i).getNumPoints();
                    go.lengths[j] = p.getNumInteriorRing() + 1;
                }
            }
        }
        if (go.ends != null)
            go.endsOffset = Feature.createEndsVector(builder, go.ends);
        if (go.lengths != null)
            go.lengthsOffset = Feature.createLengthsVector(builder, go.lengths);

        return go;
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Feature feature, HeaderMeta headerMeta) {
        GeometryFactory factory = new GeometryFactory();
        int xyLength = feature.xyLength();
        Coordinate[] coordinates = new Coordinate[xyLength >> 1];
        int c = 0;
        for (int i = 0; i < xyLength; i = i + 2)
            coordinates[c++] = new Coordinate(feature.xy(i), feature.xy(i + 1));

        IntFunction<Polygon> makePolygonWithRings = (int endsLength) -> {
            LinearRing[] lrs = new LinearRing[endsLength];
            int s = 0;
            for (int i = 0; i < endsLength; i++) {
                int e = (int) feature.ends(i);
                Coordinate[] cs = Arrays.copyOfRange(coordinates, s, e);
                lrs[i] = factory.createLinearRing(cs);
                s = e;
            }
            LinearRing shell = lrs[0];
            LinearRing holes[] = Arrays.copyOfRange(lrs, 1, endsLength);
            return factory.createPolygon(shell, holes);
        };

        Supplier<Polygon> makePolygon = () -> {
            int endsLength = feature.endsLength();
            if (endsLength > 1)
                return makePolygonWithRings.apply(endsLength);
            else
                return factory.createPolygon(coordinates);
        };

        switch (headerMeta.geometryType) {
        case GeometryType.Point:
            return factory.createPoint(coordinates[0]);
        case GeometryType.MultiPoint:
            return factory.createMultiPointFromCoords(coordinates);
        case GeometryType.LineString:
            return factory.createLineString(coordinates);
        case GeometryType.MultiLineString: {
            int lengthLengths = feature.endsLength();
            if (lengthLengths < 2)
                return factory.createMultiLineString(new LineString[] { factory.createLineString(coordinates) });
            LineString[] lss = new LineString[lengthLengths];
            int s = 0;
            for (int i = 0; i < lengthLengths; i++) {
                int e = (int) feature.ends(i);
                Coordinate[] cs = Arrays.copyOfRange(coordinates, s, e);
                lss[i] = factory.createLineString(cs);
                s = e;
            }
            return factory.createMultiLineString(lss);
        }
        case GeometryType.Polygon:
            return makePolygon.get();
        case GeometryType.MultiPolygon: {
            int lengthsLength = feature.lengthsLength();
            if (lengthsLength > 1) {
                Polygon[] ps = new Polygon[lengthsLength];
                int s = 0;
                int o = 0;
                for (int j = 0; j < lengthsLength; j++) {
                    int l = (int) feature.lengths(j);
                    LinearRing[] lrs = new LinearRing[l];
                    for (int i = 0; i < l; i++) {
                        int e = (int) feature.ends(o + i);
                        Coordinate[] cs = Arrays.copyOfRange(coordinates, s, e);
                        lrs[i] = factory.createLinearRing(cs);
                        s = e;
                    }
                    o += l;
                    LinearRing shell = lrs[0];
                    LinearRing holes[] = Arrays.copyOfRange(lrs, 1, l);
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
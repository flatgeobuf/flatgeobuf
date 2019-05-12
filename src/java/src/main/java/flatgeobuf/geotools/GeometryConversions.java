package flatgeobuf.geotools;

import java.io.IOException;
import java.util.stream.Stream;
import java.util.Arrays;
import java.util.stream.DoubleStream;

import com.google.flatbuffers.FlatBufferBuilder;

import org.locationtech.jts.geom.Coordinate;
import org.locationtech.jts.geom.GeometryFactory;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.LinearRing;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;

import flatgeobuf.generated.*;

public class GeometryConversions {
    public static int serialize(FlatBufferBuilder builder, org.locationtech.jts.geom.Geometry geometry, int geometryType, int dimensions) throws IOException {
        Stream<Coordinate> cs = Stream.of(geometry.getCoordinates());
        double[] coords;
        if (dimensions == 4)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ(), c.getM())).toArray();
        else if (dimensions == 3)
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y, c.getZ())).toArray();
        else
            coords = cs.flatMapToDouble(c -> DoubleStream.of(c.x, c.y)).toArray();
        int coordsOffset = Geometry.createCoordsVector(builder, coords);
        int[] lengths = null;
        int[] ringLengths = null;
        int[] ringCounts = null;
        if (geometryType == GeometryType.MultiLineString) {
            MultiLineString mls = (MultiLineString) geometry;
            if (mls.getNumGeometries() > 1) {
                lengths = new int[mls.getNumGeometries()];
                for (int i = 0; i < mls.getNumGeometries(); i++)
                    lengths[i] = mls.getGeometryN(i).getNumPoints() * dimensions;
            }
        } else if (geometryType == GeometryType.Polygon) {
            Polygon p = (Polygon) geometry;
            ringLengths = new int[p.getNumInteriorRing() + 1];
            ringLengths[0] = p.getExteriorRing().getNumPoints() * dimensions;
            for (int i = 0; i < p.getNumInteriorRing(); i++)
                ringLengths[i + 1] = p.getInteriorRingN(i).getNumPoints() * dimensions;
        } else if (geometryType == GeometryType.MultiPolygon) {
            MultiPolygon mp = (MultiPolygon) geometry;
            if (mp.getNumGeometries() == 1){
                Polygon p = (Polygon) mp.getGeometryN(0);
                ringLengths = new int[p.getNumInteriorRing() + 1];
                ringLengths[0] = p.getExteriorRing().getNumPoints() * dimensions;
                for (int i = 0; i < p.getNumInteriorRing(); i++)
                    ringLengths[i + 1] = p.getInteriorRingN(i).getNumPoints() * dimensions;
            } else {
                lengths = new int[mp.getNumGeometries()];
                ringCounts = new int[mp.getNumGeometries()];
                ringLengths = new int[10000];
                int c = 0;
                for (int j = 0; j < mp.getNumGeometries(); j++) {
                    Polygon p = (Polygon) mp.getGeometryN(j);
                    int ringCount = 0;
                    int ringLength = p.getExteriorRing().getNumPoints() * dimensions;
                    ringLengths[c++] = ringLength;
                    ringCount++;
                    int length = ringLength;
                    for (int i = 0; i < p.getNumInteriorRing(); i++) {
                        ringLength = p.getInteriorRingN(i).getNumPoints() * dimensions;
                        ringLengths[c++] = ringLength;
                        length += ringLength;
                        ringCount++;
                    }
                    lengths[j] = length;
                    ringCounts[j] = ringCount;
                }
            }
        }
        int lengthsOffset = 0;
        int ringLengthsOffset = 0;
        int ringCountsOffset = 0;
        if (lengths != null)
            lengthsOffset = Geometry.createLengthsVector(builder, lengths);
        if (ringLengths != null)
            ringLengthsOffset = Geometry.createRingLengthsVector(builder, ringLengths);
        if (ringCounts != null)
            ringCountsOffset = Geometry.createRingCountsVector(builder, ringCounts);
        Geometry.startGeometry(builder);
        Geometry.addCoords(builder, coordsOffset);
        if (lengths != null)
            Geometry.addLengths(builder, lengthsOffset);
        if (ringLengths != null)
            Geometry.addRingLengths(builder, ringLengthsOffset);
        if (ringCounts != null)
            Geometry.addRingCounts(builder, ringCountsOffset);
        return Geometry.endGeometry(builder);
    }

    public static org.locationtech.jts.geom.Geometry deserialize(Geometry geometry, int geometryType, int dimensions) {
        GeometryFactory factory = new GeometryFactory();
        int coordsLength = geometry.coordsLength();
        int dimLengths = coordsLength / dimensions;
        Coordinate[] coordinates = new Coordinate[dimLengths];
        int c = 0;
        for (int i = 0; i < coordsLength; i = i + dimensions)
            coordinates[c++] = new Coordinate(geometry.coords(i), geometry.coords(i + 1));
        switch (geometryType) {
            case GeometryType.Point:
                return factory.createPoint(coordinates[0]);
            case GeometryType.MultiPoint:
                return factory.createMultiPointFromCoords(coordinates);
            case GeometryType.LineString:
                return factory.createLineString(coordinates);
            case GeometryType.MultiLineString: {
                int lengthLengths = geometry.lengthsLength();
                LineString[] lss = new LineString[lengthLengths];
                int offset = 0;
                for (int i = 0; i < lengthLengths; i++) {
                    int length = (int) geometry.lengths(i) / dimensions;
                    Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + length);
                    lss[i] = factory.createLineString(cs);
                    offset += length;
                }
                return factory.createMultiLineString(lss);
            }
            case GeometryType.Polygon: {
            	int ringLengthsLength = geometry.ringLengthsLength();
            	LinearRing[] lrs = new LinearRing[ringLengthsLength];
            	int offset = 0;
            	if (ringLengthsLength > 1) {
            		for (int i = 0; i < ringLengthsLength; i++) {
                        int ringLength = (int) geometry.ringLengths(i) / dimensions;
                        Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + ringLength);
                        lrs[i] = factory.createLinearRing(cs);
                        offset += ringLength;
                    }
            		LinearRing shell = lrs[0];
            		LinearRing holes[] = Arrays.copyOfRange(lrs, 1, ringLengthsLength);
            		return factory.createPolygon(shell, holes);
            	} else {
            		return factory.createPolygon(coordinates);            		
            	}
            }
            case GeometryType.MultiPolygon: {
            	int lengthLengths = geometry.lengthsLength();
            	if (lengthLengths > 1) {
            		Polygon[] ps = new Polygon[lengthLengths];
            		int offset = 0;
            		int roffset = 0;
            		for (int j = 0; j < lengthLengths; j++) {
            			int ringCount = (int) geometry.ringCounts(j);
                    	LinearRing[] lrs = new LinearRing[ringCount];
                		for (int i = 0; i < ringCount; i++) {
                            int ringLength = (int) geometry.ringLengths(roffset + i) / dimensions;
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
            		int ringLengthsLength = geometry.ringLengthsLength();
                	LinearRing[] lrs = new LinearRing[ringLengthsLength];
                	int offset = 0;
                	if (ringLengthsLength > 1) {
                		for (int i = 0; i < ringLengthsLength; i++) {
                            int ringLength = (int) geometry.ringLengths(i) / dimensions;
                            Coordinate[] cs = Arrays.copyOfRange(coordinates, offset, offset + ringLength);
                            lrs[i] = factory.createLinearRing(cs);
                            offset += ringLength;
                        }
                		LinearRing shell = lrs[0];
                		LinearRing holes[] = Arrays.copyOfRange(lrs, 1, ringLengthsLength);
                		return factory.createPolygon(shell, holes);
                	} else {
                		return factory.createPolygon(coordinates);            		
                	}
            	}
            }
            default:
                throw new RuntimeException("Unknown geometry type");
        }
    }
}
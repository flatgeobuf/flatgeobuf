package org.wololo.flatgeobuf;

import org.junit.Test;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.geom.Point;
import org.locationtech.jts.geom.LineString;
import org.locationtech.jts.geom.Polygon;
import org.locationtech.jts.geom.MultiPolygon;
import org.locationtech.jts.geom.MultiPoint;
import org.locationtech.jts.geom.MultiLineString;
import org.locationtech.jts.geom.CoordinateSequence;
import org.locationtech.jts.geom.GeometryFactory;
import org.wololo.flatgeobuf.GeometryConversions;
import org.wololo.flatgeobuf.generated.GeometryType;

import java.util.Date;

import static org.junit.Assert.assertEquals;

public class GeometryConversionsTest {

    @Test
    public void toGeometryType() {
        assertEquals(GeometryType.Unknown, GeometryConversions.toGeometryType(Geometry.class));
        assertEquals(GeometryType.Point, GeometryConversions.toGeometryType(Point.class));
        assertEquals(GeometryType.LineString, GeometryConversions.toGeometryType(LineString.class));
        assertEquals(GeometryType.Polygon, GeometryConversions.toGeometryType(Polygon.class));
        assertEquals(GeometryType.MultiPoint, GeometryConversions.toGeometryType(MultiPoint.class));
        assertEquals(GeometryType.MultiLineString, GeometryConversions.toGeometryType(MultiLineString.class));
        assertEquals(GeometryType.MultiPolygon, GeometryConversions.toGeometryType(MultiPolygon.class));
    }

    @Test(expected = RuntimeException.class)
    public void unknowGeometryShouldThrow() {
        GeometryConversions.toGeometryType(Date.class);
    }

    @Test
    public void toGeometryTypeWithGeometrySubClass() {
        assertEquals(GeometryType.Point, GeometryConversions.toGeometryType(MyPoint.class));
    }

    private static class MyPoint extends Point {
        public MyPoint(CoordinateSequence coordinates, GeometryFactory factory) {
            super(coordinates, factory);
        }
    }

}

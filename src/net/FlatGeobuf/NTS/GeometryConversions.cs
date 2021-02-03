using System;
using System.Linq;
using System.Collections.Generic;

using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;

using NTSGeometry = NetTopologySuite.Geometries.Geometry;

using FlatBuffers;

namespace FlatGeobuf.NTS
{
    public class GeometryOffsets {
        public uint[] ends = null;
        public VectorOffset xyOffset = default;
        public VectorOffset zOffset = default;
        public VectorOffset mOffset = default;
        public VectorOffset endsOffset = default;
        public GeometryOffsets[] gos = null;
        public GeometryType Type { get; set; }
    }

    public static class GeometryConversions {
        public static GeometryOffsets BuildGeometry(FlatBufferBuilder builder, NTSGeometry geometry, GeometryType geometryType, byte dimensions)
        {
            var go = new GeometryOffsets
            {
                Type = geometryType
            };

            if (geometry == null)
                return go;

            if (geometryType == GeometryType.MultiLineString)
            {
                uint end = 0;
                MultiLineString mls = (MultiLineString) geometry;
                if (mls.NumGeometries > 1) {
                    go.ends = new uint[mls.NumGeometries];
                    for (int i = 0; i < mls.NumGeometries; i++)
                        go.ends[i] = end += (uint) mls.Geometries[i].NumPoints;
                }
            }
            else if (geometryType == GeometryType.Polygon)
            {
                go.ends = CreateEnds(geometry as Polygon);
            }
            else if (geometryType == GeometryType.MultiPolygon)
            {
                MultiPolygon mp = (MultiPolygon) geometry;
                int numGeometries = mp.NumGeometries;
                GeometryOffsets[] gos = new GeometryOffsets[numGeometries];
                for (int i = 0; i < numGeometries; i++) {
                    Polygon p = (Polygon) mp.Geometries[i];
                    gos[i] = BuildGeometry(builder, p, GeometryType.Polygon, dimensions);
                }
                go.gos = gos;
                return go;
            }

            var xy = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            go.xyOffset = Geometry.CreateXyVectorBlock(builder, xy);

            if (dimensions == 3) {
                var z = geometry.Coordinates
                    .SelectMany(c => new double[] { c.Z })
                    .ToArray();
                go.zOffset = Geometry.CreateXyVectorBlock(builder, z);
            }

            if (dimensions == 4) {
                var m = geometry.Coordinates
                    .SelectMany(c => new double[] { c.M })
                    .ToArray();
                go.mOffset = Geometry.CreateXyVectorBlock(builder, m);
            }

            if (go.ends != null)
                go.endsOffset = Geometry.CreateEndsVectorBlock(builder, go.ends);
            return go;
        }

        static uint[] CreateEnds(Polygon polygon)
        {
            var ends = new uint[polygon.NumInteriorRings + 1];
            uint end = (uint) polygon.ExteriorRing.NumPoints;
            ends[0] = end;
            for (int i = 0; i < polygon.NumInteriorRings; i++)
                ends[i + 1] = end += (uint) polygon.InteriorRings[i].NumPoints;
            return ends;
        }

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(GeometryFactory factory, int count, int dimension, ref Geometry geometry)
        {
            var lineString = factory.CreateLineString(new FlatGeobufGeometryCoordinateSequence(count, dimension, ref geometry));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(GeometryFactory factory, int count, int dimension, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufMultiLineStringSinglePart(factory, count, dimension, ref geometry);
            int start = 0;
            var lineStrings = new LineString[geometry.EndsLength];
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var end = (int) geometry.Ends(i);
                var lineString = factory.CreateLineString(new FlatGeobufGeometryCoordinateSequence(end - start, dimension, ref geometry, start));
                lineStrings[i] = lineString;
                start = end;
            }
            return factory.CreateMultiLineString(lineStrings);
        }

        static Polygon ParseFlatbufPolygonSingleRing(GeometryFactory factory, int count, int dimension, ref Geometry geometry)
        {
            var shell = factory.CreateLinearRing(new FlatGeobufGeometryCoordinateSequence(count, dimension, ref geometry));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(GeometryFactory factory, int count, int dimension, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufPolygonSingleRing(factory, count, dimension, ref geometry);
            LinearRing shell = null;
            var holes = new LinearRing[geometry.EndsLength - 1];
            int start = 0;
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var end = (int) geometry.Ends(i);
                var linearRing = factory.CreateLinearRing(new FlatGeobufGeometryCoordinateSequence(end - start, dimension, ref geometry, start));
                if (i == 0)
                    shell = linearRing;
                else
                    holes[i - 1] = linearRing;
                start = end;
            }
            return factory.CreatePolygon(shell, holes);
        }

        public static NTSGeometry FromFlatbuf(ref Geometry geometry, ref Header header)
        {
            return FromFlatbuf(ref geometry, header.GeometryType, ref header);
        }

        public static NTSGeometry FromFlatbuf(ref Geometry geometry, GeometryType type, ref Header header)
        {
            var factory = new GeometryFactory();

            if (type == GeometryType.Unknown)
                type = geometry.Type;

            switch (type)
            {
                case GeometryType.MultiPolygon:
                    int partsLength = geometry.PartsLength;
                    Polygon[] polygons = new Polygon[partsLength];
                    for (int i = 0; i < geometry.PartsLength; i++)
                    {
                        var part = geometry.Parts(i).Value;
                        polygons[i] = (Polygon) FromFlatbuf(ref part, GeometryType.Polygon, ref header);
                    }
                    return factory.CreateMultiPolygon(polygons);
            }

            int count = geometry.XyLength / 2;

            int dimension;
            if (header.HasZ)
                dimension = 3;
            else if (header.HasZ && header.HasM)
                dimension = 4;
            else
                dimension = 2;

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(new FlatGeobufGeometryCoordinateSequence(count, dimension, ref geometry)),
                GeometryType.MultiPoint => factory.CreateMultiPoint(new FlatGeobufGeometryCoordinateSequence(count, dimension, ref geometry)),
                GeometryType.LineString => factory.CreateLineString(new FlatGeobufGeometryCoordinateSequence(count, dimension, ref geometry)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(factory, count, dimension, ref geometry),
                GeometryType.Polygon => ParseFlatbufPolygon(factory, count, dimension, ref geometry),
                _ => throw new ApplicationException("FromFlatbuf: Unsupported geometry type"),
            };
        }

        public static GeometryType ToGeometryType(NTSGeometry geometry)
        {
            return geometry switch
            {
                Point _ => GeometryType.Point,
                MultiPoint _ => GeometryType.MultiPoint,
                LineString _ => GeometryType.LineString,
                MultiLineString _ => GeometryType.MultiLineString,
                Polygon _ => GeometryType.Polygon,
                MultiPolygon _ => GeometryType.MultiPolygon,
                _ => throw new ApplicationException("Unknown or null geometry"),
            };
        }
    }
}

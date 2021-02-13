using System;
using System.Linq;

using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;

using NTSGeometry = NetTopologySuite.Geometries.Geometry;

using FlatBuffers;
using NetTopologySuite;

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
        public static GeometryOffsets BuildGeometry(FlatBufferBuilder builder, NTSGeometry geometry, GeometryType geometryType, ref Header header)
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
                    gos[i] = BuildGeometry(builder, p, GeometryType.Polygon, ref header);
                }
                go.gos = gos;
                return go;
            }

            var xy = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            go.xyOffset = Geometry.CreateXyVectorBlock(builder, xy);

            if (header.HasZ) {
                var z = geometry.Coordinates
                    .SelectMany(c => new double[] { c.Z })
                    .ToArray();
                go.zOffset = Geometry.CreateXyVectorBlock(builder, z);
            }

            if (header.HasM) {
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

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(GeometryFactory factory, ref Header header, ref Geometry geometry)
        {
            var lineString = factory.CreateLineString(new FlatGeobufCoordinateSequence(ref header, ref geometry));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(GeometryFactory factory, ref Header header, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufMultiLineStringSinglePart(factory, ref header, ref geometry);
            var lineStrings = new LineString[geometry.EndsLength];
            for (var i = 0; i < geometry.EndsLength; i++)
                lineStrings[i] = factory.CreateLineString(new FlatGeobufCoordinateSequence(ref header, ref geometry, i));
            return factory.CreateMultiLineString(lineStrings);
        }

        static Polygon ParseFlatbufPolygonSingleRing(GeometryFactory factory, ref Header header, ref Geometry geometry)
        {
            var shell = factory.CreateLinearRing(new FlatGeobufCoordinateSequence(ref header, ref geometry));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(GeometryFactory factory, ref Header header, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufPolygonSingleRing(factory, ref header, ref geometry);
            LinearRing shell = null;
            var holes = new LinearRing[geometry.EndsLength - 1];
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var linearRing = factory.CreateLinearRing(new FlatGeobufCoordinateSequence(ref header, ref geometry, i));
                if (i == 0)
                    shell = linearRing;
                else
                    holes[i - 1] = linearRing;
            }
            return factory.CreatePolygon(shell, holes);
        }

        public static NTSGeometry FromFlatbuf(ref Geometry geometry, ref Header header)
        {
            return FromFlatbuf(ref geometry, header.GeometryType, ref header);
        }

        public static CoordinateSequence ToCoordinateSequence(ref Geometry geometry, ref Header header)
        {
            int count = geometry.XyLength / 2;
            var xy = geometry.GetXyBytes();
            var sequenceFactory = NtsGeometryServices.Instance.DefaultCoordinateSequenceFactory;
            if (sequenceFactory is FlatGeobufCoordinateSequenceFactory)
            {
                return new FlatGeobufCoordinateSequence(ref header, ref geometry);
            }
            else if (sequenceFactory is RawCoordinateSequenceFactory)
            {
                var offsets = new (int sourceIndex, int offset)[]
                {
                    (0, 0),
                    (0, 1)
                };
                int measures = 0;
                return new RawCoordinateSequence(new Memory<double>[] { xy.ToArray().AsMemory<double>() }, offsets, measures);
            }
            else if (sequenceFactory is DotSpatialAffineCoordinateSequenceFactory dpFactory)
            {
                return dpFactory.Create(xy.ToArray());
            }
            else
            {
                var cs = new Coordinate[count];
                if (!header.HasZ && !header.HasM)
                {
                    for (int i = 0; i < count; i++)
                        cs[i] = new Coordinate(xy[i * 2], xy[i * 2 + 1]);
                }
                else if (header.HasZ && !header.HasM)
                {
                    var z = geometry.GetZBytes();
                    for (int i = 0; i < count; i++)
                        cs[i] = new CoordinateZ(xy[i * 2], xy[i * 2 + 1], z[i]);
                }
                else if (!header.HasZ && header.HasM)
                {
                    var m = geometry.GetZBytes();
                    for (int i = 0; i < count; i++)
                        cs[i] = new CoordinateM(xy[i * 2], xy[i * 2 + 1], m[i]);
                }
                else if (header.HasZ && header.HasM)
                {
                    var z = geometry.GetZBytes();
                    var m = geometry.GetZBytes();
                    for (int i = 0; i < count; i++)
                        cs[i] = new CoordinateZM(xy[i * 2], xy[i * 2 + 1], z[i], m[i]);
                }
                return sequenceFactory.Create(cs);
            }
        }

        public static NTSGeometry FromFlatbuf(ref Geometry geometry, GeometryType type, ref Header header)
        {
            var factory = NtsGeometryServices.Instance.CreateGeometryFactory();

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

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(new FlatGeobufCoordinateSequence(ref header, ref geometry)),
                GeometryType.MultiPoint => factory.CreateMultiPoint(new FlatGeobufCoordinateSequence(ref header, ref geometry)),
                GeometryType.LineString => factory.CreateLineString(ToCoordinateSequence(ref geometry, ref header)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(factory, ref header, ref geometry),
                GeometryType.Polygon => ParseFlatbufPolygon(factory, ref header, ref geometry),
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

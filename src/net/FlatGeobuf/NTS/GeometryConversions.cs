using System;
using System.Linq;
using NetTopologySuite.Geometries;
using NTSGeometry = NetTopologySuite.Geometries.Geometry;
using Google.FlatBuffers;

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

        public static CoordinateSequence GetCoordinateSequence(NTSGeometry geometry)
        {
            return geometry switch
            {
                Point p => p.CoordinateSequence,
                MultiPoint mp => (mp.Geometries[0] as Point).CoordinateSequence,
                LineString ls => ls.CoordinateSequence,
                MultiLineString mls => (mls.Geometries[0] as LineString).CoordinateSequence,
                Polygon p => p.Shell.CoordinateSequence,
                MultiPolygon mp => (mp.Geometries[0] as Polygon).Shell.CoordinateSequence,
                _ => throw new ApplicationException("Unknown or null geometry"),
            };
        }

        public static GeometryOffsets BuildGeometry(FlatBufferBuilder builder, NTSGeometry geometry, GeometryType geometryType, HeaderT header)
        {
            var go = new GeometryOffsets
            {
                Type = geometryType
            };

            if (geometry == null)
                return go;

            var seq = GetCoordinateSequence(geometry);

            if (geometryType == GeometryType.MultiLineString)
            {
                uint end = 0;
                MultiLineString mls = (MultiLineString) geometry;
                if (mls.NumGeometries > 1)
                {
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
                for (int i = 0; i < numGeometries; i++)
                {
                    Polygon p = (Polygon) mp.Geometries[i];
                    gos[i] = BuildGeometry(builder, p, GeometryType.Polygon, header);
                }
                go.gos = gos;
                return go;
            }

            if (seq is FlatGeobufCoordinateSequence fbSeq)
            {
                go.xyOffset = Geometry.CreateXyVectorBlock(builder, fbSeq.XY);
                if (header.HasZ)
                    go.zOffset = Geometry.CreateXyVectorBlock(builder, fbSeq.Z);
                if (header.HasM)
                    go.mOffset = Geometry.CreateXyVectorBlock(builder, fbSeq.M);
            }
            else
            {
                var xy = geometry.Coordinates
                    .SelectMany(c => new double[] { c.X, c.Y })
                    .ToArray();
                go.xyOffset = Geometry.CreateXyVectorBlock(builder, xy);
                if (header.HasZ)
                {
                    var z = geometry.Coordinates
                        .SelectMany(c => new double[] { c.Z })
                        .ToArray();
                    go.zOffset = Geometry.CreateXyVectorBlock(builder, z);
                }
                if (header.HasM)
                {
                    var m = geometry.Coordinates
                        .SelectMany(c => new double[] { c.M })
                        .ToArray();
                    go.mOffset = Geometry.CreateXyVectorBlock(builder, m);
                }
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

        static MultiPoint ParseFlatbufMultiPoint(GeometryFactory factory, HeaderT header, ref Geometry geometry)
        {
            var xy = geometry.GetXyArray();
            var z = header.HasZ ? geometry.GetZArray() : null;
            var m = header.HasM ? geometry.GetMArray() : null;
            var count = xy.Length / 2;
            var points = new Point[count];
            for (int i = 0; i < count; i++)
            {
                var pxy = new double[] { xy[i * 2], xy[i * 2 + 1] };
                var pz = header.HasZ ? new double[] { z[i] } : null;
                var pm = header.HasM ? new double[] { m[i] } : null;
                points[i] = factory.CreatePoint(new FlatGeobufCoordinateSequence(pxy, pz, pm, 1, 0));
            }
            return factory.CreateMultiPoint(points);
        }

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, HeaderT header, ref Geometry geometry)
        {
            var lineString = factory.CreateLineString(seqFactory.Create(header, ref geometry));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, HeaderT header, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufMultiLineStringSinglePart(factory, seqFactory, header, ref geometry);
            var lineStrings = new LineString[geometry.EndsLength];
            for (var i = 0; i < geometry.EndsLength; i++)
                lineStrings[i] = factory.CreateLineString(seqFactory.Create(header, ref geometry, i));
            return factory.CreateMultiLineString(lineStrings);
        }

        static Polygon ParseFlatbufPolygonSingleRing(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, HeaderT header, ref Geometry geometry)
        {
            var shell = factory.CreateLinearRing(seqFactory.Create(header, ref geometry));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, HeaderT header, ref Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufPolygonSingleRing(factory, seqFactory, header, ref geometry);
            LinearRing shell = null;
            var holes = new LinearRing[geometry.EndsLength - 1];
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var linearRing = factory.CreateLinearRing(seqFactory.Create(header, ref geometry, i));
                if (i == 0)
                    shell = linearRing;
                else
                    holes[i - 1] = linearRing;
            }
            return factory.CreatePolygon(shell, holes);
        }

        static MultiPolygon ParseFlatbufMultiPolygon(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, HeaderT header, ref Geometry geometry)
        {
            int partsLength = geometry.PartsLength;
            Polygon[] polygons = new Polygon[partsLength];
            for (int i = 0; i < geometry.PartsLength; i++)
            {
                var part = geometry.Parts(i).Value;
                polygons[i] = (Polygon) FromFlatbuf(factory, seqFactory, ref part, GeometryType.Polygon, header);
            }
            return factory.CreateMultiPolygon(polygons);
        }

        public static NTSGeometry FromFlatbuf(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, ref Geometry geometry, HeaderT header)
        {
            return FromFlatbuf(factory, seqFactory, ref geometry, header.GeometryType, header);
        }

        public static CoordinateSequence ToCoordinateSequence(GeometryFactory factory, ref Geometry geometry, HeaderT header)
        {
            var sequenceFactory = factory.CoordinateSequenceFactory;
            if (sequenceFactory is FlatGeobufCoordinateSequenceFactory fbFactory)
                return fbFactory.Create(header, ref geometry);

            throw new Exception("Unexpected CoordinateSequenceFactory");

            // NOTE: below was used to compare performance with alternative CoordinateSequence

            /*int count = geometry.XyLength / 2;

            if (sequenceFactory is RawCoordinateSequenceFactory)
            {
                var offsets = new (int sourceIndex, int offset)[]
                {
                    (0, 0),
                    (0, 1)
                };
                int measures = 0;
                var memory = new CastingMemoryManager<double>(geometry.GetXyMemory().ToArray()).Memory;
                return new RawCoordinateSequence(new Memory<double>[] { memory }, offsets, measures);
            }

            var xy = geometry.GetXyArray();

            if (sequenceFactory is DotSpatialAffineCoordinateSequenceFactory dpFactory)
                return dpFactory.Create(xy);

            var cs = new Coordinate[count];
            if (!header.HasZ && !header.HasM)
            {
                for (int i = 0; i < count; i++)
                    cs[i] = new Coordinate(xy[i * 2], xy[i * 2 + 1]);
            }
            else if (header.HasZ && !header.HasM)
            {
                var z = geometry.GetZArray();
                for (int i = 0; i < count; i++)
                    cs[i] = new CoordinateZ(xy[i * 2], xy[i * 2 + 1], z[i]);
            }
            else if (!header.HasZ && header.HasM)
            {
                var m = geometry.GetMArray();
                for (int i = 0; i < count; i++)
                    cs[i] = new CoordinateM(xy[i * 2], xy[i * 2 + 1], m[i]);
            }
            else if (header.HasZ && header.HasM)
            {
                var z = geometry.GetZArray();
                var m = geometry.GetMArray();
                for (int i = 0; i < count; i++)
                    cs[i] = new CoordinateZM(xy[i * 2], xy[i * 2 + 1], z[i], m[i]);
            }
            return sequenceFactory.Create(cs);*/
        }

        public static NTSGeometry FromFlatbuf(GeometryFactory factory, FlatGeobufCoordinateSequenceFactory seqFactory, ref Geometry geometry, GeometryType type, HeaderT header)
        {
            //var factory = NtsGeometryServices.Instance.CreateGeometryFactory();

            if (type == GeometryType.Unknown)
                type = geometry.Type;

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(ToCoordinateSequence(factory, ref geometry, header)),
                GeometryType.MultiPoint => ParseFlatbufMultiPoint(factory, header, ref geometry),
                GeometryType.LineString => factory.CreateLineString(ToCoordinateSequence(factory, ref geometry, header)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(factory, seqFactory, header, ref geometry),
                GeometryType.Polygon => ParseFlatbufPolygon(factory, seqFactory, header, ref geometry),
                GeometryType.MultiPolygon => ParseFlatbufMultiPolygon(factory, seqFactory, header, ref geometry),
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

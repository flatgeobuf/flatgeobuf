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

    public struct ParseContext {
        public uint[] ends;
        public double[] xy;
        public double[] z;
        public double[] m;
        public DotSpatialAffineCoordinateSequenceFactory sequenceFactory;
        public GeometryFactory factory;
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
            go.xyOffset = Geometry.CreateXyVector(builder, xy);

            if (dimensions == 3) {
                var z = geometry.Coordinates
                    .SelectMany(c => new double[] { c.Z })
                    .ToArray();
                go.zOffset = Geometry.CreateXyVector(builder, z);
            }

            if (dimensions == 4) {
                var m = geometry.Coordinates
                    .SelectMany(c => new double[] { c.M })
                    .ToArray();
                go.mOffset = Geometry.CreateXyVector(builder, m);
            }

            if (go.ends != null)
                go.endsOffset = Geometry.CreateEndsVector(builder, go.ends);
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

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(ref ParseContext context)
        {
            var lineString = context.factory.CreateLineString(context.sequenceFactory.Create(context.xy, context.z, context.m));
            return context.factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(ref ParseContext context)
        {
            if (context.ends == null)
                return ParseFlatbufMultiLineStringSinglePart(ref context);
            var factory = new GeometryFactory(context.sequenceFactory);
            int offset = 0;
            int lastEnd = 0;
            var lineStrings = new LineString[context.ends.Length];
            for (var i = 0; i < context.ends.Length; i++)
            {
                var end = (int) context.ends[i] - lastEnd;
                var xyPart = context.xy.AsSpan().Slice(offset, end * 2).ToArray();
                var zPart = context.z?.AsSpan().Slice(offset, end).ToArray();
                var mPart = context.m?.AsSpan().Slice(offset, end).ToArray();
                var lineString = factory.CreateLineString(context.sequenceFactory.Create(xyPart, zPart, mPart));
                lineStrings[i] = lineString;
                offset += end;
                lastEnd = (int) context.ends[i];
            }
            return factory.CreateMultiLineString(lineStrings.ToArray());
        }

        static Polygon ParseFlatbufPolygonSingleRing(ref ParseContext context) {
            var factory = new GeometryFactory(context.sequenceFactory);
            var shell = context.factory.CreateLinearRing(context.sequenceFactory.Create(context.xy, context.z, context.m));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(ref ParseContext context)
        {
            if (context.ends == null)
                return ParseFlatbufPolygonSingleRing(ref context);
            var linearRings = new LinearRing[context.ends.Length];
            int offset = 0;
            int lastEnd = 0;
            for (var i = 0; i < context.ends.Length; i++)
            {
                var end = (int) context.ends[i] - lastEnd;
                var xyPart = context.xy.AsSpan().Slice(offset, end * 2).ToArray();
                var zPart = context.z?.AsSpan().Slice(offset, end).ToArray();
                var mPart = context.m?.AsSpan().Slice(offset, end).ToArray();
                var linearRing = context.factory.CreateLinearRing(context.sequenceFactory.Create(xyPart, zPart, mPart));
                linearRings[i] = linearRing;
                offset += end;
                lastEnd = (int) context.ends[i];
            }
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return context.factory.CreatePolygon(shell, holes);
        }

        public static NTSGeometry FromFlatbuf(Geometry geometry, Header header)
        {
            return FromFlatbuf(geometry, header.GeometryType, header);
        }

        public static NTSGeometry FromFlatbuf(Geometry geometry, GeometryType type, Header header)
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
                        polygons[i] = (Polygon) FromFlatbuf(geometry.Parts(i).Value, GeometryType.Polygon, header);
                    return factory.CreateMultiPolygon(polygons);
            }

            Ordinates ordinates;
            if (header.HasZ)
                ordinates = Ordinates.XYZ;
            else if (header.HasM)
                ordinates = Ordinates.XYM;
            else if (header.HasZ && header.HasM)
                ordinates = Ordinates.XYZM;
            else
                ordinates = Ordinates.XY;

            var context = new ParseContext() {
                ends = geometry.GetEndsArray(),
                xy = geometry.GetXyArray(),
                z = geometry.GetZArray(),
                m = geometry.GetMArray(),
                sequenceFactory = new DotSpatialAffineCoordinateSequenceFactory(ordinates),
                factory = factory
            };

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(context.sequenceFactory.Create(context.xy, context.z, context.m)),
                GeometryType.MultiPoint => factory.CreateMultiPoint(context.sequenceFactory.Create(context.xy, context.z, context.m)),
                GeometryType.LineString => factory.CreateLineString(context.sequenceFactory.Create(context.xy, context.z, context.m)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(ref context),
                GeometryType.Polygon => ParseFlatbufPolygon(ref context),
                _ => throw new ApplicationException("FromFlatbuf: Unsupported geometry type"),
            };
        }

        static Ordinates ConvertDimensions(byte dimensions)
        {
            return dimensions switch
            {
                1 => Ordinates.X,
                2 => Ordinates.XY,
                3 => Ordinates.XYZ,
                4 => Ordinates.XYZM,
                _ => Ordinates.XY,
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

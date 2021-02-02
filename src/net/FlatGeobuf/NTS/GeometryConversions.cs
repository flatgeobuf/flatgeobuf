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

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(GeometryFactory factory, DotSpatialAffineCoordinateSequenceFactory sequenceFactory, in Geometry geometry)
        {
            var lineString = factory.CreateLineString(CreateCoordinateSequence(sequenceFactory, geometry));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(GeometryFactory factory, DotSpatialAffineCoordinateSequenceFactory sequenceFactory, in Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufMultiLineStringSinglePart(factory, sequenceFactory, geometry);
            int offset = 0;
            int lastEnd = 0;
            var lineStrings = new LineString[geometry.EndsLength];
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var end = (int) geometry.Ends(i) - lastEnd;
                var xyPart = geometry.GetXyBytes().Slice(offset * 2, end * 2).ToArray();
                var zPart = geometry.ZLength != 0 ? geometry.GetZBytes().Slice(offset, end).ToArray() : null;
                var mPart = geometry.MLength != 0 ? geometry.GetMBytes().Slice(offset, end).ToArray() : null;
                var lineString = factory.CreateLineString(sequenceFactory.Create(xyPart, zPart, mPart));
                lineStrings[i] = lineString;
                offset += end;
                lastEnd = (int) geometry.Ends(i);
            }
            return factory.CreateMultiLineString(lineStrings.ToArray());
        }

        static Polygon ParseFlatbufPolygonSingleRing(GeometryFactory factory, DotSpatialAffineCoordinateSequenceFactory sequenceFactory, in Geometry geometry)
        {
            var shell = factory.CreateLinearRing(CreateCoordinateSequence(sequenceFactory, geometry));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(GeometryFactory factory, DotSpatialAffineCoordinateSequenceFactory sequenceFactory, in Geometry geometry)
        {
            if (geometry.EndsLength == 0)
                return ParseFlatbufPolygonSingleRing(factory, sequenceFactory, geometry);
            var linearRings = new LinearRing[geometry.EndsLength];
            int offset = 0;
            int lastEnd = 0;
            for (var i = 0; i < geometry.EndsLength; i++)
            {
                var end = (int) geometry.Ends(i) - lastEnd;
                var xyPart = geometry.GetXyBytes().Slice(offset * 2, end * 2).ToArray();
                var zPart = geometry.ZLength != 0 ? geometry.GetZBytes().Slice(offset, end).ToArray() : null;
                var mPart = geometry.MLength != 0 ? geometry.GetMBytes().Slice(offset, end).ToArray() : null;
                var linearRing = factory.CreateLinearRing(sequenceFactory.Create(xyPart, zPart, mPart));
                linearRings[i] = linearRing;
                offset += end;
                lastEnd = (int) geometry.Ends(i);
            }
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return factory.CreatePolygon(shell, holes);
        }

        public static NTSGeometry FromFlatbuf(in Geometry geometry, in Header header)
        {
            return FromFlatbuf(geometry, header.GeometryType, header);
        }

        public static CoordinateSequence CreateCoordinateSequence(DotSpatialAffineCoordinateSequenceFactory factory, in Geometry geometry)
        {
            return factory.Create(geometry.GetXyArray(), geometry.GetZArray(), geometry.GetMArray());
        }

        public static NTSGeometry FromFlatbuf(in Geometry geometry, GeometryType type, in Header header)
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

            var sequenceFactory = new DotSpatialAffineCoordinateSequenceFactory(ordinates);

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(CreateCoordinateSequence(sequenceFactory, geometry)),
                GeometryType.MultiPoint => factory.CreateMultiPoint(CreateCoordinateSequence(sequenceFactory, geometry)),
                GeometryType.LineString => factory.CreateLineString(CreateCoordinateSequence(sequenceFactory, geometry)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(factory, sequenceFactory, geometry),
                GeometryType.Polygon => ParseFlatbufPolygon(factory, sequenceFactory, geometry),
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

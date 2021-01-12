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
        public VectorOffset coordsOffset = default;
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

            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            go.coordsOffset = Geometry.CreateXyVector(builder, coordinates);

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

        static MultiLineString ParseFlatbufMultiLineStringSinglePart(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var lineString = factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static MultiLineString ParseFlatbufMultiLineString(uint[] ends, double[] coords, byte dimensions)
        {
            if (ends == null)
                return ParseFlatbufMultiLineStringSinglePart(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var coordsSpan = coords.AsSpan();

            List<LineString> lineStrings = new List<LineString>();
            uint offset = 0;
            for (var i = 0; i < ends.Length; i++)
            {
                var end = ends[i] << 1;
                var lineStringCoords = coordsSpan.Slice((int) offset, (int) (end - offset)).ToArray();
                var lineString = factory.CreateLineString(sequenceFactory.Create(lineStringCoords, dimensions));
                lineStrings.Add(lineString);
                offset = end;
            }
            return factory.CreateMultiLineString(lineStrings.ToArray());
        }

        static Polygon ParseFlatbufPolygonSingleRing(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var shell = factory.CreateLinearRing(sequenceFactory.Create(coords, dimensions));
            return factory.CreatePolygon(shell);
        }

        static Polygon ParseFlatbufPolygon(uint[] ends, double[] coords, byte dimensions)
        {
            if (ends == null)
                return ParseFlatbufPolygonSingleRing(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var linearRings = new List<LinearRing>();
            uint offset = 0;
            for (var i = 0; i < ends.Length; i++)
            {
                var end = ends[i] << 1;
                var ringCoords = coords.Skip((int) offset).Take((int) end).ToArray();
                var linearRing = factory.CreateLinearRing(sequenceFactory.Create(ringCoords, dimensions));
                linearRings.Add(linearRing);
                offset = end;
            }
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return factory.CreatePolygon(shell, holes);
        }

        static MultiPolygon ParseFlatbufMultiPolygon(uint[] lengths, uint[] ringLengths, uint[] ringCounts, double[] coords, byte dimensions)
        {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var polygons = new List<Polygon>();
            if (lengths == null)
            {
                var polygon = ParseFlatbufPolygon(ringLengths, coords, dimensions);
                polygons.Add(polygon);
            }
            else
            {
                var arraySegment = new ArraySegment<double>(coords);
                uint offset = 0;
                uint ringOffset = 0;
                for (int i = 0; i < lengths.Length; i++)
                {
                    var length = lengths[i];
                    var ringCount = ringCounts[i];
                    uint[] ringLengthSubset = null;
                    if (ringCount > 1)
                        ringLengthSubset = new ArraySegment<uint>(ringLengths).Skip((int) ringOffset).Take((int) ringCount).ToArray();
                    ringOffset += ringCount;

                    var linearRingCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                    var polygon = ParseFlatbufPolygon(ringLengthSubset, linearRingCoords, dimensions);
                    polygons.Add(polygon);
                    offset += length;
                }
            }

            return factory.CreateMultiPolygon(polygons.ToArray());
        }

        public static NTSGeometry FromFlatbuf(Geometry geometry, GeometryType type) {
            byte dimensions = 2;
            var factory = new GeometryFactory();

            if (type == GeometryType.Unknown)
                type = geometry.Type;

            switch (type)
            {
                case GeometryType.MultiPolygon:
                    int partsLength = geometry.PartsLength;
                    Polygon[] polygons = new Polygon[partsLength];
                    for (int i = 0; i < geometry.PartsLength; i++)
                        polygons[i] = (Polygon) FromFlatbuf(geometry.Parts(i).Value, GeometryType.Polygon);
                    return factory.CreateMultiPolygon(polygons);
            }

            var coords = geometry.GetXyArray();
            var ends = geometry.GetEndsArray();
            var sequenceFactory = new PackedCoordinateSequenceFactory();

            return type switch
            {
                GeometryType.Point => factory.CreatePoint(sequenceFactory.Create(coords, dimensions)),
                GeometryType.MultiPoint => factory.CreateMultiPoint(sequenceFactory.Create(coords, dimensions)),
                GeometryType.LineString => factory.CreateLineString(sequenceFactory.Create(coords, dimensions)),
                GeometryType.MultiLineString => ParseFlatbufMultiLineString(ends, coords, dimensions),
                GeometryType.Polygon => ParseFlatbufPolygon(ends, coords, dimensions),
                //case GeometryType.MultiPolygon:
                //    return ParseFlatbufMultiPolygon(ends, coords, dimensions);
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
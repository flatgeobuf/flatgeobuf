using System;
using System.Linq;
using System.Collections.Generic;

using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;
using GeoAPI.Geometries;

using FlatBuffers;

namespace FlatGeobuf.NTS
{
    public class GeometryOffsets {
        public VectorOffset? coordsOffset = null;
        public VectorOffset? lengthsOffset = null;
        public VectorOffset? ringLengthsOffset = null;
        public VectorOffset? ringCountsOffset = null;
    }

    public static class GeometryConversions {
        public static GeometryOffsets BuildGeometry(FlatBufferBuilder builder, IGeometry geometry, GeometryType geometryType, byte dimensions)
        {
            var go = new GeometryOffsets();

            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            go.coordsOffset = Geometry.CreateCoordsVector(builder, coordinates);

            var lengths = CreateLengths(geometry, dimensions);
            if (lengths != null)
                go.lengthsOffset = Geometry.CreateLengthsVector(builder, lengths.ToArray());

            var ringLengths = CreateRingLengths(geometry, geometryType, dimensions);
            if ((geometryType == GeometryType.Polygon && (geometry as IPolygon).InteriorRings.Length > 0) ||
                (geometryType == GeometryType.MultiPolygon))
                go.ringLengthsOffset = Geometry.CreateRingLengthsVector(builder, ringLengths.ToArray());
            
            if (geometryType == GeometryType.MultiPolygon && geometry.NumGeometries > 1)
            {
                var mp = geometry as IMultiPolygon;
                var ringCounts = mp.Geometries
                    .Select(g => g as IPolygon)
                    .Select(p => (uint) p.InteriorRings.Length + 1);
                go.ringCountsOffset = Geometry.CreateRingCountsVector(builder, ringCounts.ToArray());
            }

            return go;
        }

        static IEnumerable<uint> CreateRingLengths(IGeometry geometry, GeometryType geometryType, uint dimensions)
        {
            if (geometryType == GeometryType.Polygon)
            {
                IPolygon polygon = geometry as IPolygon;
                var rings = new[] { polygon.ExteriorRing }.Concat(polygon.InteriorRings);
                var ringLengths = rings
                    .Select(r => dimensions * (uint) r.Coordinates.Length);
                return ringLengths;
            }
            else if (geometryType == GeometryType.MultiPolygon)
            {
                return (geometry as IMultiPolygon).Geometries.SelectMany(g => CreateRingLengths(g, GeometryType.Polygon, dimensions));
            }
            return null;
        }

        static IEnumerable<uint> CreateLengths(IGeometry geometry, uint dimensions)
        {
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1)
            {
                var gc = geometry as IGeometryCollection;
                var lengths = gc.Geometries
                    .Select(g => dimensions * (uint) g.Coordinates.Length);
                return lengths;
            }
            return null;
        }

        static IMultiLineString ParseFlatbufMultiLineStringSinglePart(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var lineString = factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        static IMultiLineString ParseFlatbufMultiLineString(uint[] lengths, double[] coords, byte dimensions)
        {
            if (lengths == null)
                return ParseFlatbufMultiLineStringSinglePart(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);

            IList<ILineString> lineStrings = new List<ILineString>();
            uint offset = 0;
            foreach (var length in lengths)
            {
                var lineStringCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                var lineString = factory.CreateLineString(sequenceFactory.Create(lineStringCoords, dimensions));
                lineStrings.Add(lineString);
                offset += length;
            }
            return factory.CreateMultiLineString(lineStrings.ToArray());
        }

        static IPolygon ParseFlatbufPolygonSingleRing(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var shell = factory.CreateLinearRing(sequenceFactory.Create(coords, dimensions));
            return factory.CreatePolygon(shell);
        }

        static IPolygon ParseFlatbufPolygon(uint[] ringLengths, double[] coords, byte dimensions)
        {
            if (ringLengths == null)
                return ParseFlatbufPolygonSingleRing(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);
            var linearRings = new List<ILinearRing>();
            uint offset = 0;
            foreach (var length in ringLengths)
            {
                var ringCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                var linearRing = factory.CreateLinearRing(sequenceFactory.Create(ringCoords, dimensions));
                linearRings.Add(linearRing);
                offset += length;
            }
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return factory.CreatePolygon(shell, holes);
        }

        static IMultiPolygon ParseFlatbufMultiPolygon(uint[] lengths, uint[] ringLengths, uint[] ringCounts, double[] coords, byte dimensions)
        {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var polygons = new List<IPolygon>();
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
                    {
                        var ringLengthsSegment = new ArraySegment<uint>(ringLengths).Skip((int) ringOffset).Take((int) ringCount).ToArray();
                        ringLengthSubset = ringLengths;
                    }
                    ringOffset += ringCount;
                        
                    var linearRingCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                    var polygon = ParseFlatbufPolygon(ringLengthSubset, linearRingCoords, dimensions);
                    polygons.Add(polygon);
                    offset += length;
                }
            }
            
            return factory.CreateMultiPolygon(polygons.ToArray());
        }

        public static IGeometry FromFlatbuf(Feature feature, GeometryType type, byte dimensions) {
            var coords = feature.GetCoordsArray();
            var lengths = feature.GetLengthsArray();
            var ringLengths = feature.GetRingLengthsArray();
            var ringCounts = feature.GetRingCountsArray();
            var sequenceFactory = new PackedCoordinateSequenceFactory();

            var factory = new GeometryFactory();

            switch(type) {
                case GeometryType.Point:
                    return factory.CreatePoint(sequenceFactory.Create(coords, dimensions));
                case GeometryType.MultiPoint:
                    return factory.CreateMultiPoint(sequenceFactory.Create(coords, dimensions));
                case GeometryType.LineString:
                    return factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
                case GeometryType.MultiLineString:
                    return ParseFlatbufMultiLineString(lengths, coords, dimensions);
                case GeometryType.Polygon:
                    return ParseFlatbufPolygon(ringLengths, coords, dimensions);
                case GeometryType.MultiPolygon:
                    return ParseFlatbufMultiPolygon(lengths, ringLengths, ringCounts, coords, dimensions);
                default: throw new ApplicationException("FromFlatbuf: Unsupported geometry type");
            }
        }
        
        static Ordinates ConvertDimensions(byte dimensions)
        {   
            switch (dimensions)
            {
                case 1: return Ordinates.X;
                case 2: return Ordinates.XY;
                case 3: return Ordinates.XYZ;
                case 4: return Ordinates.XYZM;
                default: return Ordinates.XY;
            }
        }

        public static GeometryType ToGeometryType(IGeometry geometry)
        {
            switch(geometry)
            {
                case IPoint _:
                    return GeometryType.Point;
                case IMultiPoint _:
                    return GeometryType.MultiPoint;
                case ILineString _:
                    return GeometryType.LineString;
                case IMultiLineString _:
                    return GeometryType.MultiLineString;
                case IPolygon _:
                    return GeometryType.Polygon;
                case IMultiPolygon _:
                    return GeometryType.MultiPolygon;
                default:
                    throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
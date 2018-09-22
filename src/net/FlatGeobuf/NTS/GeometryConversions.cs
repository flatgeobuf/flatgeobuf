using System;
using System.Linq;
using System.Collections.Generic;

using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;
using GeoAPI.Geometries;

using FlatBuffers;

namespace FlatGeobuf.NTS
{
    public static class GeometryConversions {
        public static Offset<Geometry> BuildGeometry(FlatBufferBuilder builder, IGeometry geometry)
        {
            // TODO: introspect?
            uint dimensions = 2;

            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coordsOffset = Geometry.CreateCoordsVector(builder, coordinates);

            var types = CreateTypes(geometry, dimensions);
            VectorOffset? typesOffset = null;
            if (types != null)
                typesOffset = Geometry.CreateTypesVector(builder, types.ToArray());
            
            var lengths = CreateLengths(geometry, dimensions);
            VectorOffset? lengthsOffset = null;
            if (lengths != null)
                lengthsOffset = Geometry.CreateLengthsVector(builder, lengths.ToArray());

            var ringLengths = CreateRingLengths(geometry, dimensions);
            VectorOffset? ringLengthsOffset = null;
            if (ringLengths != null)
                ringLengthsOffset = Geometry.CreateRingLengthsVector(builder, ringLengths.ToArray());
            
            VectorOffset? ringCountsOffset = null;
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1 &&
                (geometry as IGeometryCollection).Geometries.Any(g => g is IPolygon))
            {
                var gc = geometry as IGeometryCollection;
                var ringCounts = gc.Geometries
                    .Where(g => g is IPolygon)
                    .Select(g => g as IPolygon)
                    .Select(p => (uint) p.InteriorRings.Length + 1);
                ringCountsOffset = Geometry.CreateRingCountsVector(builder, ringCounts.ToArray());
            }

            Geometry.StartGeometry(builder);
            if (typesOffset.HasValue)
                Geometry.AddTypes(builder, typesOffset.Value);
            if (lengthsOffset.HasValue)
                Geometry.AddLengths(builder, lengthsOffset.Value);
            if (ringLengthsOffset.HasValue)
                Geometry.AddRingLengths(builder, ringLengthsOffset.Value);
            if (ringCountsOffset.HasValue)
                Geometry.AddRingCounts(builder, ringCountsOffset.Value);

            Geometry.AddCoords(builder, coordsOffset);
            var offset = Geometry.EndGeometry(builder);

            return offset;
        }

        static IEnumerable<uint> CreateRingLengths(IGeometry geometry, uint dimensions)
        {
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1)
            {
                var gc = geometry as IGeometryCollection;
                var lengths = gc.Geometries
                    .Where(g => g is IPolygon)
                    .Select(g => g as IPolygon)
                    .Where(p => p.InteriorRings.Length > 0)
                    .Select(p => new[] { p.ExteriorRing }.Concat(p.InteriorRings))
                    .Select(rs => rs.Select(r => dimensions * (uint) r.Coordinates.Length))
                    .SelectMany(rls => rls)
                    .ToList();
                if (lengths.Count > 0)
                    return lengths;
            }
            else if (geometry is IPolygon || (geometry is IGeometryCollection && geometry.GetGeometryN(0) is IPolygon))
            {
                IPolygon polygon;
                if (geometry is IPolygon)
                    polygon = geometry as IPolygon;
                else
                    polygon = geometry.GetGeometryN(0) as IPolygon;
                if (polygon.InteriorRings.Length > 0)
                {
                    var rings = new[] { polygon.ExteriorRing }.Concat(polygon.InteriorRings);
                    var ringLengths = rings
                        .Select(r => dimensions * (uint) r.Coordinates.Length);
                    return ringLengths;
                }
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

        static IEnumerable<GeometryType> CreateTypes(IGeometry geometry, uint dimensions)
        {
            if (geometry.OgcGeometryType == OgcGeometryType.GeometryCollection)
            {
                var gc = geometry as IGeometryCollection;
                var types = gc.Geometries
                    .Select(g => ToGeometryType(g));
                return types;
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

        public static IGeometry FromFlatbuf(Geometry flatbufGeometry, GeometryType type, byte dimensions) {
            var coords = flatbufGeometry.GetCoordsArray();
            var lengths = flatbufGeometry.GetLengthsArray();
            var ringLengths = flatbufGeometry.GetRingLengthsArray();
            var ringCounts = flatbufGeometry.GetRingCountsArray();
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
                case IGeometryCollection _:
                    return GeometryType.GeometryCollection;
                default:
                    throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
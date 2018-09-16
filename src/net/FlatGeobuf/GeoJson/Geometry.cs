using System;
using System.Linq;
using System.Collections.Generic;

using NetTopologySuite.IO;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;
using GeoAPI.Geometries;

using FlatBuffers;
using FlatGeobuf;

using static FlatGeobuf.Geometry;
using static FlatGeobuf.GeometryType;

namespace FlatGeobuf.GeoJson
{
    public static class Geometry {
        public static Offset<FlatGeobuf.Geometry> BuildGeometry(FlatBufferBuilder builder, IGeometry geometry)
        {
            // TODO: find from geometry?
            uint dimensions = 2;

            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coords = CreateCoordsVector(builder, coordinates);
            
            VectorOffset? types = CreateTypesVector(builder, geometry, dimensions);
            VectorOffset? lengths = CreateLengthsVector(builder, geometry, dimensions);
            VectorOffset? ringLengths = CreateRingLengthsVector(builder, geometry, dimensions);
            
            StartGeometry(builder);
            AddType(builder, ConvertType(geometry));
            if (types.HasValue) 
                AddTypes(builder, types.Value);
            if (lengths.HasValue)
                AddLengths(builder, lengths.Value);
            if (ringLengths.HasValue)
                AddRingLengths(builder, ringLengths.Value);
            
            AddCoords(builder, coords);
            var offset = EndGeometry(builder);

            return offset;
        }

        private static VectorOffset? CreateRingLengthsVector(FlatBufferBuilder builder, IGeometry geometry, uint dimensions)
        {
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1)
            {
                // TODO: fixme, should calc ring ends only if inner ring exists
                var gc = geometry as IGeometryCollection;
                var endsArray = gc.Geometries
                    .Select(g => dimensions * (uint) g.Coordinates.Length)
                    .Take(gc.NumGeometries - 1)
                    .ToArray();
                var ends = FlatGeobuf.Geometry.CreateRingLengthsVector(builder, endsArray);
                return ends;
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
                    var ringEndsArray = rings
                        .Select(r => dimensions * (uint) r.Coordinates.Length)
                        .Take(polygon.InteriorRings.Length)
                        .ToArray();
                    var ringLengths = FlatGeobuf.Geometry.CreateRingLengthsVector(builder, ringEndsArray);
                    return ringLengths;
                }
            }
            return null;
        }

        private static VectorOffset? CreateLengthsVector(FlatBufferBuilder builder, IGeometry geometry, uint dimensions)
        {
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1)
            {
                var gc = geometry as IGeometryCollection;
                var lengthsArray = gc.Geometries
                    .Select(g => dimensions * (uint) g.Coordinates.Length)
                    .ToArray();
                var lengths = FlatGeobuf.Geometry.CreateLengthsVector(builder, lengthsArray);
                return lengths;
            }
            return null;
        }

        private static VectorOffset? CreateTypesVector(FlatBufferBuilder builder, IGeometry geometry, uint dimensions)
        {
            if (geometry.OgcGeometryType == OgcGeometryType.GeometryCollection)
            {
                var gc = geometry as IGeometryCollection;
                var typesArray = gc.Geometries
                    .Select(g => ConvertType(g))
                    .ToArray();
                var types = FlatGeobuf.Geometry.CreateTypesVector(builder, typesArray);
                return types;
            }
            return null;
        }

        private static IMultiLineString ParseFlatbufMultiLineStringSinglePart(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var lineString = factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        private static IMultiLineString ParseFlatbufMultiLineString(uint[] lengths, double[] coords, byte dimensions)
        {
            if (lengths == null)
                return ParseFlatbufMultiLineStringSinglePart(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);

            IList<ILineString> lineStrings = new List<ILineString>();
            uint offset = 0;
            for (int i = 0; i < lengths.Length; i++)
            {
                var length = lengths[i];
                var lineStringCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                var lineString = factory.CreateLineString(sequenceFactory.Create(lineStringCoords, dimensions));
                lineStrings.Add(lineString);
                offset += length;
            }
            /*
            var lineStrings = new List<uint>() { 0 }
                .Concat(new List<uint>(ends))
                .Concat(new List<uint>() { (uint) coords.Length })
                .Pairwise((s, e) => arraySegment.Skip((int) s).Take((int) e))
                .Select(cs => factory.CreateLineString(sequenceFactory.Create(cs.ToArray(), dimensions)))
                .ToArray();
            */
            return factory.CreateMultiLineString(lineStrings.ToArray());
        }

        private static IPolygon ParseFlatbufPolygonSingleRing(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var shell = factory.CreateLinearRing(sequenceFactory.Create(coords, dimensions));
            return factory.CreatePolygon(shell);
        }

        private static IPolygon ParseFlatbufPolygon(uint[] ringEnds, double[] coords, byte dimensions)
        {
            if (ringEnds == null)
                return ParseFlatbufPolygonSingleRing(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);
            var linearRings = new List<uint>() { 0 }
                .Concat(new List<uint>(ringEnds))
                .Concat(new List<uint>() { (uint) coords.Length })
                .Pairwise((s, e) => arraySegment.Skip((int) s).Take((int) e))
                .Select(cs => factory.CreateLinearRing(sequenceFactory.Create(cs.ToArray(), dimensions)));
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return factory.CreatePolygon(shell, holes);
        }

        private static IMultiPolygon ParseFlatbufMultiPolygon(uint[] lengths, uint[] ringLengths, double[] coords, byte dimensions)
        {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            IPolygon[] polygons;
            if (lengths == null)
            {
                polygons = new[] { ParseFlatbufPolygon(ringLengths, coords, dimensions) };
            }
            else
            {
                // TODO: consider ring ends
                var arraySegment = new ArraySegment<double>(coords);
                IList<ILinearRing> linearRings = new List<ILinearRing>();
                uint offset = 0;
                for (int i = 0; i < lengths.Length; i++)
                {
                    var length = lengths[i];
                    var linearRingCoords = arraySegment.Skip((int) offset).Take((int) length).ToArray();
                    var linearRing = factory.CreateLinearRing(sequenceFactory.Create(linearRingCoords, dimensions));
                    linearRings.Add(linearRing);
                    offset += length;
                }
                polygons = linearRings
                    .Select(lr => factory.CreatePolygon(lr, null))
                    .ToArray();
            }
            
            return factory.CreateMultiPolygon(polygons);
        }

        public static IGeometry FromFlatbuf(FlatGeobuf.Geometry flatbufGeometry) {
            var type = flatbufGeometry.Type;
            var dimensions = flatbufGeometry.Dimensions;
            var coords = flatbufGeometry.GetCoordsArray();
            var ends = flatbufGeometry.GetLengthsArray();
            var ringEnds = flatbufGeometry.GetRingLengthsArray();
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
                    return ParseFlatbufMultiLineString(ends, coords, dimensions);
                case GeometryType.Polygon:
                    return ParseFlatbufPolygon(ringEnds, coords, dimensions);
                case GeometryType.MultiPolygon:
                    return ParseFlatbufMultiPolygon(ends, ringEnds, coords, dimensions);
                default: throw new ApplicationException("FromFlatbuf: Unsupported geometry type");
            }
        }
        
        public static IEnumerable<TResult> Pairwise<TSource, TResult>(this IEnumerable<TSource> source, Func<TSource, TSource, TResult> resultSelector)
        {
            TSource previous = default(TSource);

            using (var it = source.GetEnumerator())
            {
                if (it.MoveNext())
                    previous = it.Current;

                while (it.MoveNext())
                    yield return resultSelector(previous, previous = it.Current);
            }
        }

        private static Ordinates ConvertDimensions(byte dimensions)
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

        private static FlatGeobuf.GeometryType ConvertType(IGeometry geometry)
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
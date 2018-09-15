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

namespace FlatGeobuf.GeoJson
{
    public static class Geometry {
        public static FlatBuffers.Offset<FlatGeobuf.Geometry> BuildGeometry(FlatBufferBuilder builder, IGeometry geometry)
        {
            // TODO: find from geometry?
            uint dimensions = 2;

            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coords = FlatGeobuf.Geometry.CreateCoordsVector(builder, coordinates);
            
            var hasTypes = false;
            var hasEnds = false;
            //var hasRingEnds = false;

            VectorOffset types = new VectorOffset();
            VectorOffset ends = new VectorOffset();
            if (geometry is IGeometryCollection && geometry.NumGeometries > 1)
            {
                var gc = geometry as IGeometryCollection;
                var endsArray = gc.Geometries
                    .Select(g => dimensions * (uint) g.Coordinates.Length)
                    .Take(gc.NumGeometries - 1)
                    .ToArray();
                ends = FlatGeobuf.Geometry.CreateEndsVector(builder, endsArray);
                hasEnds = true;
                if (geometry.OgcGeometryType == OgcGeometryType.GeometryCollection)
                {
                    var typesArray = gc.Geometries
                    .Select(g => ConvertType(g))
                    .ToArray();
                    types = FlatGeobuf.Geometry.CreateTypesVector(builder, typesArray);
                    hasTypes = true;
                }
            }
            
            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(geometry));
            if (hasTypes) 
                FlatGeobuf.Geometry.AddTypes(builder, types);
            if (hasEnds)
                FlatGeobuf.Geometry.AddEnds(builder, ends);
            //if (hasRingEnds)
            //    FlatGeobuf.Geometry.AddRingEnds(builder, ringEnds);
            
            FlatGeobuf.Geometry.AddCoords(builder, coords);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);

            return offset;
        }

        private static IMultiLineString ParseFlatbufMultiLineStringSinglePart(double[] coords, byte dimensions) {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var lineString = factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
            return factory.CreateMultiLineString(new [] { lineString });
        }

        private static IMultiLineString ParseFlatbufMultiLineString(uint[] ends, double[] coords, byte dimensions)
        {
            if (ends == null)
                return ParseFlatbufMultiLineStringSinglePart(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);
            var lineStrings = new List<uint>() { 0 }
                .Concat(new List<uint>(ends))
                .Concat(new List<uint>() { (uint) coords.Length })
                .Pairwise((s, e) => arraySegment.Skip((int) s).Take((int) e))
                .Select(cs => factory.CreateLineString(sequenceFactory.Create(cs.ToArray(), dimensions)))
                .ToArray();
            return factory.CreateMultiLineString(lineStrings);
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

        private static IMultiPolygon ParseFlatbufMultiPolygon(uint[] ends, uint[] ringEnds, double[] coords, byte dimensions)
        {
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            IPolygon[] polygons;
            if (ends == null)
            {
                polygons = new[] { ParseFlatbufPolygon(ends, coords, dimensions) };
            }
            else
            {
                var arraySegment = new ArraySegment<double>(coords);
                polygons = new List<uint>() { 0 }
                    .Concat(new List<uint>(ends))
                    .Concat(new List<uint>() { (uint) coords.Length })
                    .Pairwise((s, e) => arraySegment.Skip((int) s).Take((int) e))
                    .Select(cs => factory.CreateLinearRing(sequenceFactory.Create(cs.ToArray(), dimensions)))
                    .Select(lr => factory.CreatePolygon(lr, null))
                    .ToArray();
            }
            
            return factory.CreateMultiPolygon(polygons);
        }

        public static IGeometry FromFlatbuf(FlatGeobuf.Geometry flatbufGeometry) {
            var type = flatbufGeometry.Type;
            var dimensions = flatbufGeometry.Dimensions;
            var coords = flatbufGeometry.GetCoordsArray();
            var ends = flatbufGeometry.GetEndsArray();
            var ringEnds = flatbufGeometry.GetRingEndsArray();
            var sequenceFactory = new PackedCoordinateSequenceFactory();

            var factory = new GeometryFactory();

            switch(type) {
                case FlatGeobuf.GeometryType.Point:
                    return factory.CreatePoint(sequenceFactory.Create(coords, dimensions));
                case FlatGeobuf.GeometryType.MultiPoint:
                    return factory.CreateMultiPoint(sequenceFactory.Create(coords, dimensions));
                case FlatGeobuf.GeometryType.LineString:
                    return factory.CreateLineString(sequenceFactory.Create(coords, dimensions));
                case FlatGeobuf.GeometryType.MultiLineString:
                    return ParseFlatbufMultiLineString(ends, coords, dimensions);
                case FlatGeobuf.GeometryType.Polygon:
                    return ParseFlatbufPolygon(ringEnds, coords, dimensions);
                case FlatGeobuf.GeometryType.MultiPolygon:
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
                    return FlatGeobuf.GeometryType.Point;
                case IMultiPoint _:
                    return FlatGeobuf.GeometryType.MultiPoint;
                case ILineString _:
                    return FlatGeobuf.GeometryType.LineString;
                case IMultiLineString _:
                    return FlatGeobuf.GeometryType.MultiLineString;
                case IPolygon _:
                    return FlatGeobuf.GeometryType.Polygon;
                case IMultiPolygon _:
                    return FlatGeobuf.GeometryType.MultiPolygon;
                case IGeometryCollection _:
                    return FlatGeobuf.GeometryType.GeometryCollection;
                default:
                    throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
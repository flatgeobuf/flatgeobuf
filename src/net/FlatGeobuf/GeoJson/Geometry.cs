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
            if (geometry.OgcGeometryType == OgcGeometryType.GeometryCollection)
                return BuildGeometryGC(builder, geometry as IGeometryCollection);
            else
                return BuildGeometrySimple(builder, geometry);
        }

        private static Offset<FlatGeobuf.Geometry> BuildGeometryGC(FlatBufferBuilder builder, IGeometryCollection gc) {
            var geometriesArray = gc.Geometries
                .Select(g => BuildGeometrySimple(builder, g))
                .ToArray();
            var geometries = FlatGeobuf.Geometry.CreateGeometriesVector(builder, geometriesArray);
            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(gc));
            FlatGeobuf.Geometry.AddGeometries(builder, geometries);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);
            return offset;
        }

        private static Offset<FlatGeobuf.Geometry> BuildGeometrySimple(FlatBufferBuilder builder, IGeometry geometry) {
            // TODO: find from geometry?
            uint dimensions = 2;
            
            var hasEnds = false;
            var hasEndss = false;
            
            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coords = FlatGeobuf.Geometry.CreateCoordsVector(builder, coordinates);

            VectorOffset ends = new VectorOffset();
            VectorOffset endss = new VectorOffset();
            if (geometry is IMultiPolygon)
            {
                var multiPolygon = geometry as IMultiPolygon;
                var endsArrays = multiPolygon.Geometries
                    .Select(g => CalcEnds(g, dimensions))
                    .ToArray();
                if (endsArrays.Length == 1 && endsArrays[0].Length > 1) // special case single part MultiPolygon with holes
                {
                    ends = FlatGeobuf.Geometry.CreateEndsVector(builder, endsArrays[0]);
                    hasEnds = true;
                }
                else if (endsArrays.Length > 1)
                {
                    /*
                    var endssArray = endsArrays
                        .Select(a => (uint) a.Length)
                        .Where(l => l > 1)
                        .ToArray();
                    */
                    endss = FlatGeobuf.Geometry.CreateEndssVector(builder, endsArrays[0]);
                    hasEndss = true;
                    //var endsArray = CalcEnds(geometry, dimensions);
                    //ends = FlatGeobuf.Geometry.CreateEndsVector(builder, endsArray);
                    //hasEnds = true;
                }
            }

            if ((geometry is IPolygon && (geometry as IPolygon).NumInteriorRings > 0) ||
                geometry is IMultiLineString && geometry.NumGeometries > 1)
            {
                var endsArray = CalcEnds(geometry, dimensions);
                ends = FlatGeobuf.Geometry.CreateEndsVector(builder, endsArray);
                hasEnds = true;
            } 

            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(geometry));
            if (hasEndss)
                FlatGeobuf.Geometry.AddEndss(builder, endss);
            if (hasEnds)
                FlatGeobuf.Geometry.AddEnds(builder, ends);
            FlatGeobuf.Geometry.AddCoords(builder, coords);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);

            return offset;
        }

        private static uint[] CalcEndss(IGeometry geometry, uint dimensions) {
            var multiPolygon = geometry as IMultiPolygon;
            return multiPolygon.Geometries
                .Select(g => CalcEnds(g, dimensions).Aggregate((a, b) => a + b))
                .ToArray();
        }

        private static uint[] CalcEnds(IGeometry geometry, uint dimensions) {
            var ends = (IList<uint>) new List<uint>();
            AddEnds(ends, geometry, dimensions);
            return ends.ToArray();
        }

        private static void AddEnds(IList<uint> ends, IGeometry geometry, uint dimensions) {
            switch (geometry)
            {
                case ILineString l:
                    ends.Add(dimensions * (uint) geometry.Coordinates.Length);
                    break;
                case IPolygon p:
                    var polygon = geometry as IPolygon;
                    ends.Add(dimensions * (uint) polygon.ExteriorRing.NumPoints);
                    foreach (var innerRing in polygon.InteriorRings)
                        AddEnds(ends, innerRing, dimensions);
                    break;
                case IGeometry m when m is IMultiLineString || m is IMultiPolygon:
                    var multiGeometry = geometry as IGeometryCollection;
                    foreach (var part in multiGeometry.Geometries)
                        AddEnds(ends, part, dimensions);
                    break;
                default:
                    throw new ApplicationException($"Unsupported type {geometry.GeometryType}");
            }
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

        private static IPolygon ParseFlatbufPolygon(uint[] ends, double[] coords, byte dimensions)
        {
            if (ends == null)
                return ParseFlatbufPolygonSingleRing(coords, dimensions);
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var factory = new GeometryFactory(sequenceFactory);
            var arraySegment = new ArraySegment<double>(coords);
            var linearRings = new List<uint>() { 0 }
                .Concat(new List<uint>(ends))
                .Pairwise((s, e) => arraySegment.Skip((int) s).Take((int) e))
                .Select(cs => factory.CreateLinearRing(sequenceFactory.Create(cs.ToArray(), dimensions)));
            var shell = linearRings.First();
            var holes = linearRings.Skip(1).ToArray();
            return factory.CreatePolygon(shell, holes);
        }

        public static IGeometry FromFlatbuf(FlatGeobuf.Geometry flatbufGeometry) {
            var type = flatbufGeometry.Type;
            var dimensions = flatbufGeometry.Dimensions;
            var coords = flatbufGeometry.GetCoordsArray();
            var ends = flatbufGeometry.GetEndsArray();
            var endss = flatbufGeometry.GetEndssArray();
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
                    return ParseFlatbufPolygon(ends, coords, dimensions);
                case FlatGeobuf.GeometryType.MultiPolygon:
                {
                    var polygons = new List<uint>();
                    if (endss == null) // special case single part MultiPolygon
                    {
                        var polygon = ParseFlatbufPolygon(ends, coords, dimensions);
                        return factory.CreateMultiPolygon(new[] { polygon });
                    }
                    else
                    {
                        var arraySegment = new ArraySegment<double>(coords);
                        var endsArraySegment = new ArraySegment<uint>(ends);
                        var polygons2 = new List<uint>() { 0 }
                            .Concat(new List<uint>(flatbufGeometry.GetEndssArray()))
                            .Pairwise((s, e) => {
                                var linearRings2 = endsArraySegment.Skip((int) s).Take((int) e)
                                    .Pairwise((s2, e2) => arraySegment.Skip((int) s2).Take((int) e2))
                                    .Select(cs => factory.CreateLinearRing(sequenceFactory.Create(cs.ToArray(), dimensions)));
                                var shell = linearRings2.First();
                                var holes = linearRings2.Skip(1).ToArray();
                                return factory.CreatePolygon(shell, holes);
                            })
                            .ToArray();
                        return factory.CreateMultiPolygon(polygons2);
                    }

                    /*
                    var ends = flatbufGeometry.GetEndsArray();

                    var ends = new List<uint>() { 0 }
                        .Concat(new List<uint>(flatbufGeometry.GetEndsArray())).ToArray();
                    var endsArraySegment = new ArraySegment<uint>(ends);
                    var arraySegment = new ArraySegment<double>(coords);

                    var polygons = new List<uint>() { 0 }
                        .Concat(new List<uint>(flatbufGeometry.GetEndssArray()))
                        .Pairwise((s, e) => {
                            var linearRings2 = endsArraySegment.Skip((int) s).Take((int) e)
                                .Pairwise((s2, e2) => arraySegment.Skip((int) s2).Take((int) e2))
                                .Select(cs => factory.CreateLinearRing(sequenceFactory.Create(cs.ToArray(), dimensions)));
                            var shell = linearRings2.First();
                            var holes = linearRings2.Skip(1).ToArray();
                            return factory.CreatePolygon(shell, holes);
                        })
                        .ToArray();

                    return factory.CreateMultiPolygon(polygons);
                    */
                }
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
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
            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coords = FlatGeobuf.Geometry.CreateCoordsVector(builder, coordinates);
            if (geometry is IMultiPolygon) {
                var endss = FlatGeobuf.Geometry.CreateEndssVector(builder, CalcEndss(geometry));
                FlatGeobuf.Geometry.AddEndss(builder, endss);
            }
            var ends = FlatGeobuf.Geometry.CreateEndsVector(builder, CalcEnds(geometry));
            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(geometry));
            FlatGeobuf.Geometry.AddEnds(builder, ends);
            FlatGeobuf.Geometry.AddCoords(builder, coords);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);
            return offset;
        }

        private static uint[] CalcEndss(IGeometry geometry) {
            if (!(geometry is IMultiPolygon))
                return null;
            var multiPolygon = geometry as IMultiPolygon;
            return multiPolygon.Geometries
                .Select(g => CalcEnds(g).Aggregate((a, b) => a + b))
                .ToArray();
        }

        private static uint[] CalcEnds(IGeometry geometry) {
            var ends = (IList<uint>) new List<uint>();
            AddEnds(ends, geometry);
            return ends.ToArray();
        }

        private static void AddEnds(IList<uint> ends, IGeometry geometry) {
            if (geometry is IPoint) {
                ends.Add(2);
            } else if (geometry is IMultiPoint || geometry is ILineString || geometry is ILinearRing) {
                ends.Add(2 * (uint) geometry.Coordinates.Length);
            } else if (geometry is IPolygon) {
                var polygon = geometry as IPolygon;
                ends.Add(2 * (uint) polygon.ExteriorRing.NumPoints);
                foreach (var innerRing in polygon.InteriorRings)
                    AddEnds(ends, innerRing);
            } else if (geometry is IMultiLineString) {
                var multiLineString = geometry as IMultiLineString;
                foreach (var lineString in multiLineString.Geometries)
                    AddEnds(ends, lineString);
            } else {
                throw new ApplicationException($"CalcLengths: Unsupported type {geometry.GeometryType}");
            }
        }

        public static IGeometry FromFlatbuf(FlatGeobuf.Geometry flatbufGeometry) {
            
            var type = flatbufGeometry.Type;
            var dimensions = flatbufGeometry.Dimensions;
            var coords = flatbufGeometry.GetCoordsArray();
            var sequenceFactory = new PackedCoordinateSequenceFactory();
            var coordinateSequence = sequenceFactory.Create(coords, dimensions);

            var factory = new GeometryFactory();

            switch(type) {
                case FlatGeobuf.GeometryType.Point:
                    return factory.CreatePoint(coordinateSequence);
                case FlatGeobuf.GeometryType.MultiPoint:
                    return factory.CreateMultiPoint(coordinateSequence);
                case FlatGeobuf.GeometryType.LineString:
                    return factory.CreateLineString(coordinateSequence);
                //case FlatGeobuf.GeometryType.MultiLineString:
                //    return factory.CreateMultiLineString(coordinateSequence);
                default: throw new ApplicationException("FromFlatbuf: Unsupported geometry type");
            }
        }

        private static FlatGeobuf.GeometryType ConvertType(IGeometry geometry)
        {
            switch(geometry) {
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
                default: throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
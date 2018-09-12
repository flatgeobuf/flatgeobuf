using System;
using System.Linq;
using System.Collections.Generic;

using NetTopologySuite.IO;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using GeoAPI.Geometries;

using FlatBuffers;
using FlatGeobuf;

namespace FlatGeobuf.GeoJson
{
    public static class Geometry {
        public static FlatBuffers.Offset<FlatGeobuf.Geometry> BuildGeometry(FlatBufferBuilder builder, IGeometry geometry)
        {
            if (geometry.OgcGeometryType == OgcGeometryType.GeometryCollection) {
                var gc = geometry as IGeometryCollection;
                gc.Geometries.Select(g)
                foreach (var part in gc.Geometries) {
                    BuildGeometry(builder, part)
                }
                
                FlatGeobuf.Geometry.CreateGeometriesVector(builder, geometries);
            }


            var coordinates = geometry.Coordinates
                .SelectMany(c => new double[] { c.X, c.Y })
                .ToArray();
            var coords = FlatGeobuf.Geometry.CreateCoordsVector(builder, coordinates);

            var coordsEndss = CalcCoordsEndss(geometry);

            CalcCoordsEndss(geometry)
                .Select(ends => ends.)

            var coordinatesEnds = CalcCoordsEndss(geometry).ToArray();
            var coordsLengths = FlatGeobuf.Geometry.CreateCoordsEndsVector(builder, coordinatesEnds);

            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(geometry));
            FlatGeobuf.Geometry.AddCoordsEnds(builder, coordsLengths);
            FlatGeobuf.Geometry.AddCoords(builder, coords);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);

            return offset;
        }

        private static IList<IList<uint>> CalcEndss(IMultiPolygon multiPolygon) {
            return multiPolygon.Geometries
                .Select(g => CalcEnds(g)).ToList();
        }

        private static IList<uint> CalcEnds(IGeometry geometry) {
            var ends = (IList<uint>) new List<uint>();
            AddEnds(ends, geometry);
            return ends;
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
            var point = flatbufGeometry.GetCoordsArray();
            var factory = new GeometryFactory();
            var geometry = factory.CreatePoint(new Coordinate(point[0], point[1]));
            return geometry;
        }

        private static FlatGeobuf.GeometryType ConvertType(IGeometry geometry)
        {
            switch(geometry) {
                case IPoint _: return FlatGeobuf.GeometryType.Point;
                case IMultiPoint _: return FlatGeobuf.GeometryType.MultiPoint;
                case ILineString _: return FlatGeobuf.GeometryType.LineString;
                case IMultiLineString _: return FlatGeobuf.GeometryType.MultiLineString;
                case IPolygon _: return FlatGeobuf.GeometryType.Polygon;
                case IMultiPolygon _: return FlatGeobuf.GeometryType.MultiPolygon;
                case IGeometryCollection _: return FlatGeobuf.GeometryType.GeometryCollection;
                default: throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
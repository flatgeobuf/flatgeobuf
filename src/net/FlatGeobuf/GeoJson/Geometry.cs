using System;
using System.Linq;

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
            var coordinates = geometry.Coordinates.Select(c => new double[] { c.X, c.Y });

            var point = coordinates.First();

            var vectorOffset = FlatGeobuf.Geometry.CreateCoordsVector(builder, point);

            FlatGeobuf.Geometry.StartGeometry(builder);
            FlatGeobuf.Geometry.AddType(builder, ConvertType(geometry));
            FlatGeobuf.Geometry.AddCoords(builder, vectorOffset);
            var offset = FlatGeobuf.Geometry.EndGeometry(builder);

            return offset;
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
                default: throw new ApplicationException("Unknown or null geometry");
            }
        }
    }
}
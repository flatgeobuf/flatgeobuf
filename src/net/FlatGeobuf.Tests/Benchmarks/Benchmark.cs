using System.Collections.Generic;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using FlatGeobuf.NTS;
using FlatGeobuf.Tests.NTS;
using NetTopologySuite.Features;

namespace FlatGeobuf.Benchmarks
{
    public class Program
    {
        public struct GeometryFixture {
            public FeatureCollection fc;
            public GeometryType geometryType;
            public byte dimensions;
            public byte[] flatgeobuf;
        }

        public class Geometry
        {
            GeometryFixture pointFixture;
            GeometryFixture polygonFixture;
            GeometryFixture pointWithAttributesFixture;

            public Geometry()
            {
                var point = GeometryRoundtripTests.MakeFeature("POINT (1.2 -2.1)");
                var polygon = GeometryRoundtripTests.MakeFeature("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))");

                var attributes = new Dictionary<string, object>()
                {
                    ["test1"] = 1,
                    ["test2"] = 1.1,
                    ["test3"] = "test",
                    ["test4"] = true,
                    ["test5"] = "teståöä2",
                    ["test6"] = false,
                };

                var pointWithAttributes = GeometryRoundtripTests.MakeFeature("POINT (1.2 -2.1)", attributes);

                pointFixture = new GeometryFixture() {
                    fc = GeometryRoundtripTests.MakeFeatureCollection(point),
                    geometryType = GeometryConversions.ToGeometryType(point.Geometry),
                    dimensions = GeometryRoundtripTests.GetDimensions(point.Geometry),
                };
                pointFixture.flatgeobuf = FeatureCollectionConversions.Serialize(pointFixture.fc, pointFixture.geometryType, pointFixture.dimensions);

                polygonFixture = new GeometryFixture() {
                    fc = GeometryRoundtripTests.MakeFeatureCollection(polygon),
                    geometryType = GeometryConversions.ToGeometryType(polygon.Geometry),
                    dimensions = GeometryRoundtripTests.GetDimensions(polygon.Geometry),
                };
                polygonFixture.flatgeobuf = FeatureCollectionConversions.Serialize(polygonFixture.fc, polygonFixture.geometryType, polygonFixture.dimensions);

                pointWithAttributesFixture = new GeometryFixture() {
                    fc = GeometryRoundtripTests.MakeFeatureCollection(pointWithAttributes),
                    geometryType = GeometryConversions.ToGeometryType(pointWithAttributes.Geometry),
                    dimensions = GeometryRoundtripTests.GetDimensions(pointWithAttributes.Geometry),
                };
                pointWithAttributesFixture.flatgeobuf = FeatureCollectionConversions.Serialize(pointWithAttributesFixture.fc, pointWithAttributesFixture.geometryType, pointWithAttributesFixture.dimensions);
            }

            [Benchmark]
            public void SerializePoint() {
                FeatureCollectionConversions.Serialize(pointFixture.fc, pointFixture.geometryType, pointFixture.dimensions);
            }

            [Benchmark]
            public void DeserializePoint() {
                FeatureCollectionConversions.Deserialize(pointFixture.flatgeobuf);
            }

            [Benchmark]
            public void SerializePolygon() {
                FeatureCollectionConversions.Serialize(polygonFixture.fc, polygonFixture.geometryType, polygonFixture.dimensions);
            }

            [Benchmark]
            public void DeserializePolygon() {
                FeatureCollectionConversions.Deserialize(polygonFixture.flatgeobuf);
            }

            [Benchmark]
            public void SerializePointWithAttributes() {
                FeatureCollectionConversions.Serialize(pointWithAttributesFixture.fc, pointWithAttributesFixture.geometryType, pointWithAttributesFixture.dimensions);
            }

            [Benchmark]
            public void DeserializePointWithAttributes() {
                FeatureCollectionConversions.Deserialize(pointWithAttributesFixture.flatgeobuf);
            }
        }

        public static void Main(string[] args) => BenchmarkSwitcher.FromAssembly(typeof(Program).Assembly).Run(args);
    }
}
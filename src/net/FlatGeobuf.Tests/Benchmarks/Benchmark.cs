using System.Collections.Generic;
using System.Linq;
using BenchmarkDotNet;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Columns;
using BenchmarkDotNet.Configs;
using BenchmarkDotNet.Jobs;
using BenchmarkDotNet.Running;
using FlatBuffers;
using FlatGeobuf.NTS;
using FlatGeobuf.Tests.NTS;
using NetTopologySuite;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;
using Perfolizer.Horology;

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

        public struct FeatureFixture {
            public FeatureCollection fc;
            public GeometryType geometryType;
            public byte dimensions;
            public byte[] flatgeobuf;
        }

        //[SimpleJob(RuntimeMoniker.CoreRt31)]
        //[SimpleJob(RuntimeMoniker.CoreRt50)]
        public class FeatureConversionRoundtripBenchmark
        {
            Header header;
            NetTopologySuite.Features.IFeature feature;
            ByteBuffer bytes;

            //[Params(2, 20, 200, 20000)]
            [Params(200)]
            //[Params(20000)]
            public int Vertices;

            //[Params("Default", "Raw", "DotSpatial", "FlatGeobuf")]
            //[Params("Default", "Raw", "DotSpatial")]
            [Params("FlatGeobuf")]
            //[Params("Raw")]
            public string Sequence;

            static public LineString MakeLineString(int maxVertices) {
                var factory = new GeometryFactory();
                var cs = Enumerable.Range(1, maxVertices).Select(x => new Coordinate(x, 100 + x)).ToArray();
                var geometry = factory.CreateLineString(cs);
                return geometry;
            }

            [GlobalSetup]
            public void Setup()
            {
                if (Sequence == "Raw")
                {
                    var ordinateGroups = new[] { Ordinates.XY };
                    NtsGeometryServices.Instance = new NtsGeometryServices(new RawCoordinateSequenceFactory(ordinateGroups), new PrecisionModel(), 0);
                }
                else if (Sequence == "DotSpatial")
                {
                    NtsGeometryServices.Instance = new NtsGeometryServices(new DotSpatialAffineCoordinateSequenceFactory(Ordinates.XY), new PrecisionModel(), 0);
                }
                else if (Sequence == "FlatGeobuf")
                {
                    NtsGeometryServices.Instance = new NtsGeometryServices(new FlatGeobufCoordinateSequenceFactory(), new PrecisionModel(), 0);
                }
                var geometryType = GeometryType.LineString;
                byte dimensions = 2;
                var headerBuffer = FeatureCollectionConversions.BuildHeader(1, geometryType, dimensions, null, null);
                headerBuffer.Position += 4;
                header = Header.GetRootAsHeader(headerBuffer);
                var geometry = MakeLineString(Vertices);
                feature = new NetTopologySuite.Features.Feature(geometry, null);
                bytes = FeatureConversions.ToByteBuffer(feature, ref header);
                bytes.Position += 4;
                feature = FeatureConversions.FromByteBuffer(bytes, ref header);
            }

            [Benchmark]
            public ByteBuffer LineStringSerialize()
            {
                return FeatureConversions.ToByteBuffer(feature, ref header);
            }

            [Benchmark]
            public int LineStringDeserialize()
            {
                var feature = FeatureConversions.FromByteBuffer(bytes, ref header);
                var ls = feature.Geometry as LineString;
                int i;
                for (i = 0; i < ls.CoordinateSequence.Count; i++) {
                    ls.CoordinateSequence.GetX(i);
                    ls.CoordinateSequence.GetY(i);
                }
                return i;
            }
        }

        public class FeatureCollectionConversionsBenchmark
        {
            GeometryFixture pointFixture;
            GeometryFixture polygonFixture;
            GeometryFixture polygonZFixture;
            GeometryFixture pointWithAttributesFixture;

            public FeatureCollectionConversionsBenchmark()
            {
                var point = GeometryRoundtripTests.MakeFeature("POINT (1.2 -2.1)");
                var polygon = GeometryRoundtripTests.MakeFeature("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))");
                var polygonZ = GeometryRoundtripTests.MakeFeature("POLYGON Z((30 10 1, 40 40 2, 20 40 3, 10 20 4, 30 10 5))");

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

                polygonZFixture = new GeometryFixture() {
                    fc = GeometryRoundtripTests.MakeFeatureCollection(polygonZ),
                    geometryType = GeometryConversions.ToGeometryType(polygonZ.Geometry),
                    dimensions = GeometryRoundtripTests.GetDimensions(polygonZ.Geometry),
                };
                polygonZFixture.flatgeobuf = FeatureCollectionConversions.Serialize(polygonZFixture.fc, polygonZFixture.geometryType, polygonZFixture.dimensions);

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
            public void SerializePolygonZ() {
                FeatureCollectionConversions.Serialize(polygonZFixture.fc, polygonZFixture.geometryType, polygonZFixture.dimensions);
            }

            [Benchmark]
            public void DeserializePolygonZ() {
                FeatureCollectionConversions.Deserialize(polygonZFixture.flatgeobuf);
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

        public static void Main(string[] args)
        {
            //var summaryStyle = new BenchmarkDotNet.Reports.SummaryStyle(null, false, SizeUnit.B, TimeUnit.Microsecond);
            //var config = DefaultConfig.Instance.WithSummaryStyle(summaryStyle);
            var config = DefaultConfig.Instance;
            //var config = new DebugInProcessConfig();
            BenchmarkSwitcher.FromAssembly(typeof(Program).Assembly).Run(args, config);
        }
    }
}
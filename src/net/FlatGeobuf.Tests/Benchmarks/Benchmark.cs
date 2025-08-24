using System.Linq;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Configs;
using BenchmarkDotNet.Running;
using Google.FlatBuffers;
using FlatGeobuf.NTS;
using NetTopologySuite;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;

namespace FlatGeobuf.Benchmarks
{
    public class Program
    {
        public struct GeometryFixture
        {
            public FeatureCollection fc;
            public GeometryType geometryType;
            public byte dimensions;
            public byte[] flatgeobuf;
        }

        public struct FeatureFixture
        {
            public FeatureCollection fc;
            public GeometryType geometryType;
            public byte dimensions;
            public byte[] flatgeobuf;
        }

        //[SimpleJob(RuntimeMoniker.CoreRt31)]
        //[SimpleJob(RuntimeMoniker.CoreRt50)]
        public class FeatureConversionRoundtripBenchmark
        {
            HeaderT header;
            NetTopologySuite.Features.IFeature feature;
            ByteBuffer bytes;
            GeometryFactory factory;
            FlatGeobufCoordinateSequenceFactory sequenceFactory;


            //[Params(2, 20, 200, 20000)]
            [Params(200)]
            //[Params(20000)]
            public int Vertices;

            //[Params("Default", "Raw", "DotSpatial", "FlatGeobuf")]
            //[Params("Default", "Raw", "DotSpatial")]
            [Params("FlatGeobuf")]
            //[Params("Raw")]
            public string Sequence;

            static public LineString MakeLineString(int maxVertices)
            {
                var factory = new GeometryFactory();
                var cs = Enumerable.Range(1, maxVertices).Select(x => new Coordinate(x, 100 + x)).ToArray();
                var geometry = factory.CreateLineString(cs);
                return geometry;
            }

            [GlobalSetup]
            public void Setup()
            {
                sequenceFactory = new FlatGeobufCoordinateSequenceFactory();
                factory = new GeometryFactory();
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
                header = Header.GetRootAsHeader(headerBuffer).UnPack();
                var geometry = MakeLineString(Vertices);
                feature = new NetTopologySuite.Features.Feature(geometry, null);
                bytes = FeatureConversions.ToByteBuffer(feature, header);
                bytes.Position += 4;
                feature = FeatureConversions.FromByteBuffer(factory, sequenceFactory, bytes, header);
            }

            [Benchmark]
            public ByteBuffer LineStringSerialize()
            {
                return FeatureConversions.ToByteBuffer(feature, header);
            }

            [Benchmark]
            public int LineStringDeserialize()
            {
                var feature = FeatureConversions.FromByteBuffer(factory, sequenceFactory, bytes, header);
                var ls = feature.Geometry as LineString;
                int i;
                for (i = 0; i < ls.CoordinateSequence.Count; i++)
                {
                    ls.CoordinateSequence.GetX(i);
                    ls.CoordinateSequence.GetY(i);
                }
                return i;
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
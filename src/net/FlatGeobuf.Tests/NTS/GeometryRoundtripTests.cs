using Microsoft.VisualStudio.TestTools.UnitTesting;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using FlatGeobuf.NTS;
using System.Collections.Generic;
using System.Threading.Tasks;
using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;

namespace FlatGeobuf.Tests.NTS
{
    [TestClass]
    public class GeometryRoundtripTests
    {
        private static readonly NetTopologySuite.NtsGeometryServices _services = new(
            CoordinateArraySequenceFactory.Instance, new PrecisionModel(), 0, GeometryOverlay.NG, new PerOrdinateEqualityComparer());

        public static NetTopologySuite.Features.Feature MakeFeature(string wkt, Dictionary<string, object> attr = null) {
            var reader = new WKTReader(_services);
            var geometry = reader.Read(wkt);
            var feature = new NetTopologySuite.Features.Feature(geometry, attr != null ? new AttributesTable(attr) : null);
            return feature;
        }

        public static FeatureCollection MakeFeatureCollection(NetTopologySuite.Features.Feature f)
        {
            var fc = new FeatureCollection
            {
                f
            };
            return fc;
        }

        public static byte GetDimensions(NetTopologySuite.Geometries.Geometry g)
        {
            byte dimensions = 2;
            if (!double.IsNaN(g.Coordinate.Z))
                dimensions += 1;
            if (!double.IsNaN(g.Coordinate.M))
                dimensions += 1;
            return dimensions;
        }

        static async Task<IFeature> ToDeserializedFeature(string wkt)
        {
            var f = MakeFeature(wkt);
            var fc = MakeFeatureCollection(f);
            var geometryType = GeometryConversions.ToGeometryType(f.Geometry);
            byte dimensions = GetDimensions(f.Geometry);
            var flatgeobuf = await FeatureCollectionConversions.SerializeAsync(fc, geometryType, dimensions);
            return FeatureCollectionConversions.Deserialize(flatgeobuf)[0];
        }
        static async Task<string> RoundTrip(string wkt)
        {
            var f = await ToDeserializedFeature(wkt);
            var gd = GetDimensions(f.Geometry);
            return new WKTWriter(gd).Write(f.Geometry);
        }

        [TestMethod]
        [DataRow("POINT (1.2 -2.1)")]
        [DataRow("POINT (1.2 -2.1 3)")]
        [DataRow("POINT M(1.2 -2.1 4)")]
        [DataRow("POINT Z(1.2 -2.1 3)")]
        [DataRow("POINT ZM(1.2 -2.1 3 4)")]
        [DataRow("LINESTRING (1.2 -2.1, 2.2 -2.2)")]
        [DataRow("POLYGON((30 10, 40 40, 20 40, 10 20, 30 10))")]
        [DataRow("POLYGON((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30))")]
        [DataRow("MULTIPOINT ((10 40), (40 30), (20 20), (30 10))")]
        [DataRow("MULTILINESTRING((30 20, 45 40, 10 40), (40 10, 10 20, 5 10, 15 5))")]
        [DataRow("MULTIPOLYGON(((30 20, 45 40, 10 40, 30 20)), ((15 5, 40 10, 10 20, 5 10, 15 5)))")]
        public async Task GeometryCopyable(string wkt)
        {
            var actual = await ToDeserializedFeature(wkt);
            var copy = actual.Geometry.Copy();

            Assert.IsTrue(actual.Geometry.EqualsExact(copy));
        }

        [TestMethod]
        public async Task PointMutable()
        {
            var f = MakeFeature("POINT (1.2 -2.1)");
            var fc = MakeFeatureCollection(f);
            var flatgeobuf = await FeatureCollectionConversions.SerializeAsync(fc, GeometryType.Point, 2);
            var fcOut = FeatureCollectionConversions.Deserialize(flatgeobuf);
            var point = fcOut[0].Geometry as Point;
            point.CoordinateSequence.SetOrdinate(0, 0, 0.01);
            Assert.AreEqual(0.01, point.Coordinate.X);
        }

        [TestMethod]
        public async Task Point()
        {
            var expected = "POINT (1.2 -2.1)";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task MultiPoint()
        {
            var expected = "MULTIPOINT ((10 40), (40 30), (20 20), (30 10))";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task PointZ()
        {
            var expected = "POINT Z(1.2 -2.1 3.1)";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }


        [TestMethod]
        public async Task LineString()
        {
            var expected = "LINESTRING (1.2 -2.1, 2.4 -4.8)";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task LineStringZ()
        {
            var expected = "LINESTRING Z(1.2 -2.1 3.1, 2.4 -4.8 4.2)";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task Polygon()
        {
            var expected = "POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task PolygonWithHole()
        {
            var expected = "POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30))";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task PolygonZ()
        {
            var expected = "POLYGON Z((30 10 1, 40 40 2, 20 40 3, 10 20 4, 30 10 5))";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public async Task MultiPolygon()
        {
            var expected = "MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)), ((15 5, 40 10, 10 20, 5 10, 15 5)))";
            var actual = await RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }
    }
}

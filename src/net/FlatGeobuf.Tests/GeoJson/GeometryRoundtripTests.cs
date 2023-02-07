using Microsoft.VisualStudio.TestTools.UnitTesting;
using Json.Comparer;

using Newtonsoft.Json.Linq;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using System.Threading.Tasks;

namespace FlatGeobuf.Tests.GeoJson
{
    [TestClass]
    public class GeometryRoundtripTests
    {
        static string MakeFeatureCollection(string wkt) {
            return MakeFeatureCollection(new string[] { wkt });
        }

        static string MakeFeatureCollection(string[] wkts) {
            var fc = new FeatureCollection();
            foreach (var wkt in wkts)
                fc.Add(MakeFeature(wkt));
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        static NetTopologySuite.Features.Feature MakeFeature(string wkt) {
            var reader = new WKTReader();
            var geometry = reader.Read(wkt);
            var feature = new NetTopologySuite.Features.Feature(geometry, null);
            return feature;
        }

        [TestMethod]
        public async Task Point()
        {
            var expected = MakeFeatureCollection("POINT(1.2 -2.1)");
            var bytes = await GeoJsonConversions.SerializeAsync(expected);
            var actual = GeoJsonConversions.Deserialize(bytes);
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void Points()
        {
            var expected = MakeFeatureCollection(new string[] { "POINT(1.2 -2.1)", "POINT(2.4 -4.8)" });
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPoint()
        {
            var expected = MakeFeatureCollection("MULTIPOINT(10 40, 40 30, 20 20, 30 10)");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPointAlternativeSyntax()
        {
            var expected = MakeFeatureCollection("MULTIPOINT((10 40), (40 30), (20 20), (30 10))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void LineString()
        {
            var expected = MakeFeatureCollection("LINESTRING(1.2 -2.1, 2.4 -4.8)");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public async Task MultiLineString()
        {
            var expected = MakeFeatureCollection("MULTILINESTRING((10 10, 20 20, 10 40), (40 40, 30 30, 40 20, 30 10), (50 50, 60 60, 50 90))");
            var bytes = await GeoJsonConversions.SerializeAsync(expected);
            var actual = GeoJsonConversions.Deserialize(bytes);
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiLineStringSinglePart()
        {
            var expected = MakeFeatureCollection("MULTILINESTRING((1.2 -2.1, 2.4 -4.8))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void Polygon()
        {
            var expected = MakeFeatureCollection("POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPolygon()
        {
            var expected = MakeFeatureCollection("MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)), ((15 5, 40 10, 10 20, 5 10, 15 5)))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPolygonWithHole()
        {
            var expected = MakeFeatureCollection("MULTIPOLYGON (((40 40, 20 45, 45 30, 40 40)), ((20 35, 10 30, 10 10, 30 5, 45 20, 20 35), (30 20, 20 15, 20 25, 30 20)))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPolygonSinglePart()
        {
            var expected = MakeFeatureCollection("MULTIPOLYGON (((30 20, 45 40, 10 40, 30 20)))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void MultiPolygonSinglePartWithHole()
        {
            var expected = MakeFeatureCollection("MULTIPOLYGON (((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30)))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void PolygonWithHole()
        {
            var expected = MakeFeatureCollection("POLYGON ((35 10, 45 45, 15 40, 10 20, 35 10), (20 30, 35 35, 30 20, 20 30))");
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        private static void AssertJson(string expected, string actual) {
            var compare = new JTokenComparer(new IndexArrayKeySelector());
            var result = compare.Compare(JObject.Parse(expected), JObject.Parse(actual));
            Assert.AreEqual(ComparisonResult.Identical, result.ComparisonResult);
        }
    }
}

using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.Collections.Generic;

using Json.Comparer;

using Newtonsoft.Json.Linq;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using NetTopologySuite.Geometries;

namespace FlatGeobuf.Tests.GeoJson
{
    [TestClass]
    public class AttributeRoundtripTests
    {
        static string MakeFeatureCollection(IDictionary<string, object> attributes) {
            return MakeFeatureCollection(new[] { attributes });
        }

        static string MakeFeatureCollection(IDictionary<string, object>[] attributess){
            var fc = new FeatureCollection();
            foreach (var attributes in attributess)
                fc.Add(MakeFeature(attributes));
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        static NetTopologySuite.Features.Feature MakeFeature(IDictionary<string, object> attributes) {
            var attributesTable = new AttributesTable(attributes);
            var factory = new GeometryFactory();
            var point = factory.CreatePoint(new Coordinate(1,1));
            var feature = new NetTopologySuite.Features.Feature(point, attributesTable);
            return feature;
        }

        [TestMethod]
        public void Number()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void NumberTwoAttribs()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1,
                ["test2"] = 1
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void NumberFiveAttribs()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1,
                ["test2"] = 1,
                ["test3"] = 1,
                ["test4"] = 1,
                ["test5"] = 1
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }
        
        [TestMethod]
        public void NumberTenAttribs()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1,
                ["test2"] = 1,
                ["test3"] = 1,
                ["test4"] = 1,
                ["test5"] = 1,
                ["test6"] = 1,
                ["test7"] = 1,
                ["test8"] = 1,
                ["test9"] = 1,
                ["test10"] = 1
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void NumberWithDecimal()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1.1
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void Boolean()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = true
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void String()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = "test"
            };
            var expected = MakeFeatureCollection(attributes);
            var actual = GeoJsonConversions.Deserialize(GeoJsonConversions.Serialize(expected));
            AssertJson(expected, actual);
        }

        [TestMethod]
        public void Mixed()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test1"] = 1,
                ["test2"] = 1.1,
                ["test3"] = "test",
                ["test4"] = true,
                ["test5"] = "teståöä2",
                ["test6"] = false,
            };
            var expected = MakeFeatureCollection(attributes);
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

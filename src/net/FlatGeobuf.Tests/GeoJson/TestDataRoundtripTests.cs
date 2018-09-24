using Microsoft.VisualStudio.TestTools.UnitTesting;
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Linq;
using Json.Comparer;
using Newtonsoft.Json.Linq;

namespace FlatGeobuf.Tests.GeoJson
{
    [TestClass]
    public class TestDataRoundtripTests
    {
        [TestMethod]
        public void StatesTest()
        {
            var expected = File.ReadAllText("..\\..\\..\\..\\..\\..\\test\\data\\states.geojson");
            var bytes = GeoJsonConversions.FromGeoJson(expected);
            var actual  = GeoJsonConversions.ToGeoJson(bytes);
        }

        [TestMethod]
        public void TigerRoadsTest()
        {
            var expected = File.ReadAllText("..\\..\\..\\..\\..\\..\\test\\data\\tiger_roads.geojson");
            var bytes = GeoJsonConversions.FromGeoJson(expected);
            var actual = GeoJsonConversions.ToGeoJson(bytes);
            AssertJson(expected, actual);
        }

        private void AssertJson(string expected, string actual)
        {
            var compare = new JTokenComparer(new IndexArrayKeySelector());
            var result = compare.Compare(JObject.Parse(expected), JObject.Parse(actual));
            Assert.AreEqual(ComparisonResult.Identical, result.ComparrisonResult);
        }
    }
}

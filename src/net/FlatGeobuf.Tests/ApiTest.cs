using Microsoft.VisualStudio.TestTools.UnitTesting;

using Newtonsoft.Json.Linq;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using FlatGeobuf;

namespace FlatGeobuf.Tests
{
    [TestClass]
    public class ApiTest
    {
        private string MakeFeatureCollection(string wkt) {
            var reader = new WKTReader();
            var geometry = reader.Read(wkt);
            var feature = new NetTopologySuite.Features.Feature(geometry, null);
            var fc = new FeatureCollection();
            fc.Add(feature);
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        [TestMethod]
        public void RoundtripPoint()
        {
            var expected = MakeFeatureCollection("POINT(1 1)");

            var bytes = Api.FromGeoJson(expected);
            var result = Api.ToGeoJson(bytes);

            var equals = JToken.DeepEquals(expected, result);

            Assert.IsTrue(equals);
        }
    }
}

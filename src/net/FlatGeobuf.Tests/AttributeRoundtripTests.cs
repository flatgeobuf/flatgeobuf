using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.Collections.Generic;

using Newtonsoft.Json.Linq;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using NetTopologySuite.Geometries;
using GeoAPI.Geometries;
using FlatGeobuf;

namespace FlatGeobuf.Tests
{
    [TestClass]
    public class AttributeRoundtripTests
    {
        string MakeFeatureCollection(IDictionary<string, object> attributes) {
            return MakeFeatureCollection(new[] { attributes });
        }

        string MakeFeatureCollection(IDictionary<string, object>[] attributess){
            var fc = new FeatureCollection();
            foreach (var attributes in attributess)
                fc.Add(MakeFeature(attributes));
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        NetTopologySuite.Features.Feature MakeFeature(IDictionary<string, object> attributes) {
            var reader = new GeoJsonReader();
            var attributesTable = new AttributesTable(attributes);
            var factory = new GeometryFactory();
            var point = factory.CreatePoint(new Coordinate(1,1));
            var feature = new NetTopologySuite.Features.Feature(point, attributesTable);
            return feature;
        }

        [TestMethod]
        public void Integer()
        {
            var attributes = new Dictionary<string, object>()
            {
                ["test"] = 1
            };

            var expected = MakeFeatureCollection(attributes);
            var bytes = Api.FromGeoJson(expected);
            var result = Api.ToGeoJson(bytes);
            var equals = JToken.DeepEquals(expected, result);
            Assert.IsTrue(equals);
        }

    }
}

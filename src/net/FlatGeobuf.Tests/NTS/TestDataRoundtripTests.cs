using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.IO;
using NetTopologySuite.IO;
using NetTopologySuite.Features;
using FlatGeobuf.NTS;
using GeoAPI.Geometries;
using System.Linq;

namespace FlatGeobuf.Tests.NTS
{
    [TestClass]
    public class TestDataRoundtripTests
    {
        [TestMethod]
        public void StatesTest()
        {
            var geojson = File.ReadAllText("../../../../../../test/data/states.geojson");
            var reader = new GeoJsonReader();
            var fcExpected = reader.Read<FeatureCollection>(geojson);
            var bytes = FeatureCollectionConversions.Serialize(fcExpected, GeometryType.Unknown);
            var fcActual = FeatureCollectionConversions.Deserialize(bytes);
            Assert.AreEqual(fcExpected.Count, fcActual.Count);
        }

        [TestMethod]
        public void TigerRoadsTest()
        {
            var geojson = File.ReadAllText("../../../../../../test/data/tiger_roads.geojson");
            var reader = new GeoJsonReader();
            var fcExpected = reader.Read<FeatureCollection>(geojson);
            var bytes = FeatureCollectionConversions.Serialize(fcExpected, GeometryType.LineString);
            var fcActual = FeatureCollectionConversions.Deserialize(bytes);
            Assert.AreEqual(fcExpected.Count, fcActual.Count);
        }

        [TestMethod]
        public void CountriesTest()
        {
            var bytes = File.ReadAllBytes("../../../../../../test/data/countries.fgb");
            var fcActual = FeatureCollectionConversions.Deserialize(bytes);
            Assert.AreEqual(179, fcActual.Count);

            var rect = new Envelope(12, 12, 56, 56);
            var list = FeatureCollectionConversions.Deserialize(new MemoryStream(bytes), rect).ToList();
            Assert.AreEqual(3, list.Count);

            bytes = FeatureCollectionConversions.Serialize(fcActual, GeometryType.Unknown, 2, fcActual.Count, true);
            fcActual = FeatureCollectionConversions.Deserialize(bytes);
            Assert.AreEqual(179, fcActual.Count);

            list = FeatureCollectionConversions.Deserialize(new MemoryStream(bytes), rect).ToList();
            Assert.AreEqual(3, list.Count);
        }
    }
}

using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.IO;
using NetTopologySuite.IO;
using NetTopologySuite.Features;
using FlatGeobuf.NTS;
using System.Linq;
using NetTopologySuite.Geometries;

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
        }
        
        [TestMethod]
        public void BinaryTest()
        {
            var srcBytes = File.ReadAllBytes("../../../../../../test/data/binary_wkb.fgb");
            var src = FeatureCollectionConversions.Deserialize(srcBytes);
            Assert.AreEqual(1, src.Count);

            var dstBytes = FeatureCollectionConversions.Serialize(src, GeometryType.Unknown);
            var dst = FeatureCollectionConversions.Deserialize(dstBytes);
            Assert.AreEqual(1, dst.Count);

            Assert.AreEqual("08b2681a1482afff056faced1a3aae40", src[0].Attributes["id"]);
            Assert.AreEqual(src[0].Attributes["id"], dst[0].Attributes["id"]);
            Assert.IsInstanceOfType(src[0].Attributes["wkb"], typeof(byte[]));
            Assert.IsInstanceOfType(dst[0].Attributes["wkb"], typeof(byte[]));
            byte[] srcWkb = (byte[])src[0].Attributes["wkb"];
            byte[] dstWkb = (byte[])src[0].Attributes["wkb"];
            Assert.AreEqual(21, srcWkb.Length);
            Assert.AreEqual(srcWkb.Length, dstWkb.Length);
            Assert.IsTrue(srcWkb.SequenceEqual(dstWkb));
        }
    }
}

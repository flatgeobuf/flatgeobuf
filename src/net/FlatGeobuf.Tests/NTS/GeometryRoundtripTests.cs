using Microsoft.VisualStudio.TestTools.UnitTesting;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using FlatGeobuf.NTS;

namespace FlatGeobuf.Tests.NTS
{
    [TestClass]
    public class GeometryRoundtripTests
    {
        static NetTopologySuite.Features.Feature MakeFeature(string wkt) {
            var reader = new WKTReader();
            var geometry = reader.Read(wkt);
            var feature = new NetTopologySuite.Features.Feature(geometry, null);
            return feature;
        }

        static string RoundTrip(string wkt)
        {
            var f = MakeFeature(wkt);
            var fc = new FeatureCollection
            {
                f
            };
            var geometryType = GeometryConversions.ToGeometryType(f.Geometry);
            byte dimensions = 2;
            if (double.IsNaN(f.Geometry.Coordinate.Z))
                dimensions += 1;
            if (double.IsNaN(f.Geometry.Coordinate.M))
                dimensions += 1;
            var flatgeobuf = FeatureCollectionConversions.Serialize(fc, geometryType, dimensions);
            var fcOut = FeatureCollectionConversions.Deserialize(flatgeobuf);
            var wktOut = new WKTWriter().Write(fcOut[0].Geometry);
            return wktOut;
        }

        [TestMethod]
        public void Point()
        {
            var expected = "POINT (1.2 -2.1)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        public void PointZ()
        {
            var expected = "POINT (1.2 -2.1 3.1)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }


        [TestMethod]
        public void LineString()
        {
            var expected = "LINESTRING (1.2 -2.1, 2.4 -4.8)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        /*[TestMethod]
        public void LineStringZ()
        {
            var expected = "LINESTRING (1.2 -2.1 3.1, 2.4 -4.8 4.2)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }*/

        [TestMethod]
        public void Polygon()
        {
            var expected = "POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }
    }
}

using Microsoft.VisualStudio.TestTools.UnitTesting;
using NetTopologySuite.Features;
using NetTopologySuite.IO;
using FlatGeobuf.NTS;
using System.Collections.Generic;

namespace FlatGeobuf.Tests.NTS
{
    [TestClass]
    public class GeometryRoundtripTests
    {
        public static NetTopologySuite.Features.Feature MakeFeature(string wkt, Dictionary<string, object> attr = null) {
            var reader = new WKTReader();
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

        static string RoundTrip(string wkt)
        {
            var f = MakeFeature(wkt);
            var fc = MakeFeatureCollection(f);
            var geometryType = GeometryConversions.ToGeometryType(f.Geometry);
            byte dimensions = GetDimensions(f.Geometry);
            var flatgeobuf = FeatureCollectionConversions.Serialize(fc, geometryType, dimensions);
            var fcOut = FeatureCollectionConversions.Deserialize(flatgeobuf);
            var wktOut = new WKTWriter(dimensions).Write(fcOut[0].Geometry);
            return wktOut;
        }

        [TestMethod]
        public void Point()
        {
            var expected = "POINT (1.2 -2.1)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public void PointZ()
        {
            var expected = "POINT Z(1.2 -2.1 3.1)";
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

        [TestMethod]
        public void LineStringZ()
        {
            var expected = "LINESTRING Z(1.2 -2.1 3.1, 2.4 -4.8 4.2)";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public void Polygon()
        {
            var expected = "POLYGON ((30 10, 40 40, 20 40, 10 20, 30 10))";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }

        [TestMethod]
        public void PolygonZ()
        {
            var expected = "POLYGON Z((30 10 1, 40 40 2, 20 40 3, 10 20 4, 30 10 5))";
            var actual = RoundTrip(expected);
            Assert.AreEqual(expected, actual);
        }
    }
}

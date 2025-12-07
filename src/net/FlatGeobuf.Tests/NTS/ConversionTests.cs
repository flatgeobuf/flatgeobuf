using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;

namespace FlatGeobuf.Tests.NTS;
[TestClass]
public class ConversionTests
{
    [TestMethod]
    public void DateTimeConversionTest()
    {
        var collection = new FeatureCollection
        {
            new NetTopologySuite.Features.Feature
            {
                Geometry = LineString.Empty,
                Attributes = new AttributesTable()
                {
                    { "date", new DateTime(2024, 6, 15, 12, 30, 0, DateTimeKind.Utc) },
                }
            }
        };
        var data = FlatGeobuf.NTS.FeatureCollectionConversions.Serialize(collection, GeometryType.LineString);
        var result = FlatGeobuf.NTS.FeatureCollectionConversions.Deserialize(data);
        Assert.HasCount(1, result);
        var attributes = result[0].Attributes;
        Assert.IsInstanceOfType(attributes["date"], typeof(DateTime));
    }
}
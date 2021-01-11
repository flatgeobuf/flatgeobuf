using FlatGeobuf.NTS;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using NetTopologySuite.IO;
using Newtonsoft.Json;

namespace FlatGeobuf
{
    public static class GeoJsonConversions
    {
        public static byte[] Serialize(string geojson, byte dimensions = 2)
        {
            var reader = dimensions == 2 ? new GeoJsonReader() 
                            : new GeoJsonReader(new GeometryFactory(new PrecisionModel(), 4326), new JsonSerializerSettings(), dimensions);
            var fc = reader.Read<FeatureCollection>(geojson);
            var bytes = FeatureCollectionConversions.Serialize(fc, GeometryType.Unknown, dimensions);
            return bytes;
        }

        public static string Deserialize(byte[] bytes, byte dimensions = 2)
        {
            var fc = FeatureCollectionConversions.Deserialize(bytes, dimensions);
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc, dimensions);
            return geojson;
        }
    }
}

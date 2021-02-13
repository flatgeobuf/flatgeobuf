using System.Threading.Tasks;
using FlatGeobuf.NTS;
using NetTopologySuite.Features;
using NetTopologySuite.IO;

namespace FlatGeobuf
{
    public static class GeoJsonConversions
    {
        public static byte[] Serialize(string geojson)
        {
            var reader = new GeoJsonReader();
            var fc = reader.Read<FeatureCollection>(geojson);
            var bytes = FeatureCollectionConversions.Serialize(fc, GeometryType.Unknown);
            return bytes;
        }

        public static async Task<byte[]> SerializeAsync(string geojson)
        {
            var reader = new GeoJsonReader();
            var fc = reader.Read<FeatureCollection>(geojson);
            var bytes = await FeatureCollectionConversions.SerializeAsync(fc, GeometryType.Unknown);
            return bytes;
        }

        public static string Deserialize(byte[] bytes)
        {
            var fc = FeatureCollectionConversions.Deserialize(bytes);
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }
    }
}

using FlatGeobuf.NTS;
using NetTopologySuite.Features;
using NetTopologySuite.IO;

namespace FlatGeobuf
{
    public static class GeoJsonConversions
    {
        public static byte[] FromGeoJson(string geojson)
        {
            var reader = new GeoJsonReader();
            var fc = reader.Read<FeatureCollection>(geojson);
            var bytes = FeatureCollectionConversions.ToFlatGeobuf(fc, GeometryType.Unknown);
            return bytes;
        }

        public static string ToGeoJson(byte[] bytes)
        {
            var fc = FeatureCollectionConversions.FromFlatGeobuf(bytes);
            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }
    }
}

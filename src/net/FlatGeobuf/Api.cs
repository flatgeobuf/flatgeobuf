using System;

using FlatBuffers;
using FlatGeobuf.GeoJson;

namespace FlatGeobuf
{
    public static class Api
    {
        public static byte[] FromGeoJson(string geojson)
        {
            var bytes = GeoJsonFeatureCollection.ToFlatGeobuf(geojson);
            return bytes;
        }

        public static string ToGeoJson(byte[] bytes)
        {
            var geojson = GeoJsonFeatureCollection.FromFlatGeobuf(bytes);
            return geojson;
        }
    }
}

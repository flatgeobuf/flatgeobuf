using NetTopologySuite.Features;
using NetTopologySuite.IO;
using System.IO;

namespace FlatGeobuf
{
    public class FlatGeobufConverter
    {
        public static void Main(string[] args)
        {
            var inFile = args[0];
            var outFile = args[1];

            var geojson = File.ReadAllText(inFile);

            var reader = new GeoJsonReader();
            var fc = reader.Read<FeatureCollection>(geojson);
            var bytes = FlatGeobuf.NTS.FeatureCollectionConversions.Serialize(fc, GeometryType.Unknown);

            File.WriteAllBytes(outFile, bytes);
        }
    }
}

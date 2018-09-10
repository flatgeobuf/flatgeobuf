using NetTopologySuite.IO;
using NetTopologySuite.Features;

using FlatBuffers;
using FlatGeobuf;

namespace FlatGeobuf.GeoJson
{
    public static class Feature {
        public static byte[] ToByteBuffer(IFeature feature) {
            var builder = new FlatBufferBuilder(40);

            var geometryOffset = Geometry.BuildGeometry(builder, feature.Geometry);

            FlatGeobuf.Feature.StartFeature(builder);
            FlatGeobuf.Feature.AddGeometry(builder, geometryOffset);
            var offset = FlatGeobuf.Feature.EndFeature(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }

        public static IFeature FromByteBuffer(ByteBuffer bb) {
            var feature = FlatGeobuf.Feature.GetRootAsFeature(bb);
            var geometry = Geometry.FromFlatbuf(feature.Geometry.Value);
            var f = new NetTopologySuite.Features.Feature(geometry, null);
            return f;
        }
    }
}
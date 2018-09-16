using System;
using System.Linq;
using System.IO;
using NetTopologySuite.IO;

using FlatBuffers;
using FlatGeobuf;

namespace FlatGeobuf.GeoJson
{
    public static class FeatureCollection {
        public static byte[] ToFlatGeobuf(string geojson) {
            var reader = new GeoJsonReader();
            var fc = reader.Read<NetTopologySuite.Features.FeatureCollection>(geojson);

            if (fc.Features.Count == 0)
                throw new ApplicationException("Empty feature collection is not allowed as input");

            var header = BuildHeader(fc);

            var memoryStream = new MemoryStream();
            memoryStream.Write(header, 0, header.Length);

            foreach (var feature in fc.Features)
            {
                var buffer = FlatGeobuf.GeoJson.Feature.ToByteBuffer(feature);
                memoryStream.Write(buffer, 0, buffer.Length);
            }
            
            return memoryStream.ToArray();
        }

        public static string FromFlatGeobuf(byte[] bytes) {
            var fc = new NetTopologySuite.Features.FeatureCollection();

            var bb = new FlatBuffers.ByteBuffer(bytes);
            
            var headerLength = ByteBufferUtil.GetSizePrefix(bb);
            bb.Position = FlatBufferConstants.SizePrefixLength;
            var header = Header.GetRootAsHeader(bb);

            var count = header.FeaturesCount;
            bb.Position += headerLength;

            while (count-- > 0) {
                var featureLength = ByteBufferUtil.GetSizePrefix(bb);
                bb.Position += FlatBufferConstants.SizePrefixLength;
                var feature = Feature.FromByteBuffer(bb);
                fc.Add(feature);
                bb.Position += featureLength;
            }

            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        private static byte[] BuildHeader(NetTopologySuite.Features.FeatureCollection fc) {
            var builder = new FlatBufferBuilder(40);

            // TODO: optionally use first feature as column schema
            var feature = fc.Features.First();
            VectorOffset? columnsOffset = null;
            if (feature.Attributes != null && feature.Attributes.Count > 0) {
                var columns = feature.Attributes.GetNames()
                    .Select(n => Column.CreateColumn(builder, builder.CreateString(n), ColumnType.STRING))
                    .ToArray();
                columnsOffset = Header.CreateColumnsVector(builder, columns);
            }

            Header.StartHeader(builder);
            if (columnsOffset.HasValue)
                Header.AddColumns(builder, columnsOffset.Value);
            Header.AddFeaturesCount(builder, (ulong) fc.Features.Count);
            var offset = Header.EndHeader(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }
    }
}
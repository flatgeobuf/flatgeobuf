using System;
using System.Collections.Generic;
using System.Linq;
using System.IO;
using NetTopologySuite.IO;

using FlatBuffers;

namespace FlatGeobuf.GeoJson
{
    public class ColumnMeta
    {
        public string Name { get; set; }
        public ColumnType Type { get; set; }
    }

    public static class GeoJsonFeatureCollection {
        public static byte[] ToFlatGeobuf(string geojson) {
            var reader = new GeoJsonReader();
            var fc = reader.Read<NetTopologySuite.Features.FeatureCollection>(geojson);

            if (fc.Features.Count == 0)
                throw new ApplicationException("Empty feature collection is not allowed as input");

            // TODO: make it optional to use first feature as column schema
            var featureFirst = fc.Features.First();
            IList<ColumnMeta> columns = null;
            if (featureFirst.Attributes != null && featureFirst.Attributes.Count > 0)
            {
                columns = featureFirst.Attributes.GetNames()
                    .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
                    .ToList();
            }

            var header = BuildHeader(fc, columns);

            var memoryStream = new MemoryStream();
            memoryStream.Write(header, 0, header.Length);

            foreach (var feature in fc.Features)
            {
                var buffer = GeoJsonFeature.ToByteBuffer(feature, columns);
                memoryStream.Write(buffer, 0, buffer.Length);
            }
            
            return memoryStream.ToArray();
        }

        private static ColumnType ToColumnType(Type type) {
            switch (Type.GetTypeCode(type)) {
                case TypeCode.Byte: return ColumnType.UByte;
                case TypeCode.SByte: return ColumnType.Byte;
                case TypeCode.Boolean: return ColumnType.Bool;
                case TypeCode.Int32: return ColumnType.Int;
                case TypeCode.Int64: return ColumnType.Long;
                case TypeCode.Double: return ColumnType.Double;
                case TypeCode.String: return ColumnType.String;
                default: throw new ApplicationException("Unknown type");
            }
        }

        public static string FromFlatGeobuf(byte[] bytes) {
            var fc = new NetTopologySuite.Features.FeatureCollection();

            var bb = new ByteBuffer(bytes);
            
            var headerLength = ByteBufferUtil.GetSizePrefix(bb);
            bb.Position = FlatBufferConstants.SizePrefixLength;
            var header = Header.GetRootAsHeader(bb);
            
            var count = header.FeaturesCount;
            bb.Position += headerLength;

            while (count-- > 0) {
                var featureLength = ByteBufferUtil.GetSizePrefix(bb);
                bb.Position += FlatBufferConstants.SizePrefixLength;
                var feature = GeoJsonFeature.FromByteBuffer(bb, header);
                fc.Add(feature);
                bb.Position += featureLength;
            }

            var writer = new GeoJsonWriter();
            var geojson = writer.Write(fc);
            return geojson;
        }

        private static byte[] BuildHeader(NetTopologySuite.Features.FeatureCollection fc, IList<ColumnMeta> columns) {
            // TODO: size might not be enough, need to be adaptive
            var builder = new FlatBufferBuilder(1024);

            // TODO: make it optional to use first feature as column schema
            var feature = fc.Features.First();
            VectorOffset? columnsOffset = null;
            if (columns != null) {
                var columnsArray = columns
                    .Select(c => Column.CreateColumn(builder, builder.CreateString(c.Name), c.Type))
                    .ToArray();
                columnsOffset = Column.CreateSortedVectorOfColumn(builder, columnsArray);
            }

            Layer.StartLayer(builder);
            if (columnsOffset.HasValue)
                Layer.AddColumns(builder, columnsOffset.Value);
            Layer.AddGeometryType(builder, GeoJsonGeometry.ToGeometryType(feature.Geometry));
            var layerOffset = Layer.EndLayer(builder);
            var layerOffsets = new[] { layerOffset };
            var layersOffset = Header.CreateLayersVector(builder, layerOffsets);

            Header.StartHeader(builder);
            Header.AddLayers(builder, layersOffset);
            Header.AddFeaturesCount(builder, (ulong) fc.Features.Count);
            var offset = Header.EndHeader(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }
    }
}
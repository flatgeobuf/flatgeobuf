using System;
using System.Collections.Generic;
using System.Linq;
using System.IO;

using FlatBuffers;
using NetTopologySuite.Features;
using FlatGeobuf.Index;

namespace FlatGeobuf.NTS
{
    public class ColumnMeta
    {
        public string Name { get; set; }
        public ColumnType Type { get; set; }
    }

    public static class FeatureCollectionConversions {
        public static byte[] ToFlatGeobuf(FeatureCollection fc) {

            ulong count = (ulong) fc.Features.LongCount();

            if (count == 0)
                throw new ApplicationException("Empty feature collection is not allowed as input");

            var index = new PackedHilbertRTree(count);
            foreach (var f in fc.Features)
            {
                var b = f.Geometry.EnvelopeInternal;
                index.Add(b.MinX, b.MinY, b.MaxX, b.MaxY);
            }
            index.Finish();

            // TODO: make it optional to use first feature as column schema
            var featureFirst = fc.Features.First();
            IList<ColumnMeta> columns = null;
            if (featureFirst.Attributes != null && featureFirst.Attributes.Count > 0)
            {
                columns = featureFirst.Attributes.GetNames()
                    .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
                    .ToList();
            }

            var header = BuildHeader(fc, columns, index);

            using (var memoryStream = new MemoryStream())
            {
                memoryStream.Write(header, 0, header.Length);

                var indexBytes = index.ToBytes();
                memoryStream.Write(indexBytes, 0, indexBytes.Length);
                
                using (var offsetsStream = new MemoryStream())
                using (var offetsWriter = new BinaryWriter(offsetsStream))
                {
                    ulong offset = 0;
                    for (ulong i = 0; i < count; i++)
                    {
                        var feature = fc.Features[(int)index.Indices[i]];
                        var buffer = FeatureConversions.ToByteBuffer(feature, columns);
                        memoryStream.Write(buffer, 0, buffer.Length);
                        offetsWriter.Write(offset);
                        offset += (ulong) buffer.Length;
                    }
                    offsetsStream.WriteTo(memoryStream);
                }
                
                return memoryStream.ToArray();
            }
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

        public static FeatureCollection FromFlatGeobuf(byte[] bytes) {
            var fc = new NetTopologySuite.Features.FeatureCollection();

            var bb = new ByteBuffer(bytes);
            
            var headerLength = ByteBufferUtil.GetSizePrefix(bb);
            bb.Position = FlatBufferConstants.SizePrefixLength;
            var header = Header.GetRootAsHeader(bb);
            
            var count = header.FeaturesCount;
            var nodeSize = header.IndexNodeSize;

            bb.Position += headerLength;

            var index = new PackedHilbertRTree(count, nodeSize);
            var indexData = bytes.Skip(headerLength).Take((int) index.Size).ToArray();
            index.Load(indexData);

            bb.Position += (int) index.Size;

            while (count-- > 0) {
                var featureLength = ByteBufferUtil.GetSizePrefix(bb);
                bb.Position += FlatBufferConstants.SizePrefixLength;
                var feature = FeatureConversions.FromByteBuffer(bb, header);
                fc.Add(feature);
                bb.Position += featureLength;
            }

            return fc;
        }

        private static byte[] BuildHeader(NetTopologySuite.Features.FeatureCollection fc, IList<ColumnMeta> columns, PackedHilbertRTree index) {
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
            Layer.AddGeometryType(builder, GeometryConversions.ToGeometryType(feature.Geometry));
            var layerOffset = Layer.EndLayer(builder);
            var layerOffsets = new[] { layerOffset };
            var layersOffset = Header.CreateLayersVector(builder, layerOffsets);

            Header.StartHeader(builder);
            Header.AddLayers(builder, layersOffset);
            if (index != null)
                Header.AddIndexNodesCount(builder, index.NumNodes);
            Header.AddFeaturesCount(builder, (ulong) fc.Features.Count);
            var offset = Header.EndHeader(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }
    }
}
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

    public class LayerMeta
    {
        public string Name { get; set; }
        public GeometryType GeometryType { get; set; }
        public byte Dimensions { get; set; }
        public IList<ColumnMeta> Columns { get; set; }
    }

    public static class FeatureCollectionConversions {
        public static byte[] ToFlatGeobuf(FeatureCollection fc, GeometryType? geometryType = null, byte dimensions = 2, IList<ColumnMeta> columns = null) {
            ulong count = (ulong) fc.Features.LongCount();

            if (count == 0)
                return new byte[0];
            
            var index = new PackedHilbertRTree(count);
            foreach (var f in fc.Features)
            {
                var b = f.Geometry.EnvelopeInternal;
                index.Add(b.MinX, b.MinY, b.MaxX, b.MaxY);
            }
            index.Finish();

            var featureFirst = fc.Features.First();

            if (geometryType == null)
                geometryType = GeometryConversions.ToGeometryType(featureFirst.Geometry);

            if (columns == null && featureFirst.Attributes != null) {
                columns = featureFirst.Attributes.GetNames()
                    .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
                    .ToList();
            }

            using (var memoryStream = new MemoryStream())
            {
                using (var featuresStream = new MemoryStream())
                using (var offsetsStream = new MemoryStream())
                using (var offetsWriter = new BinaryWriter(offsetsStream))
                {
                    ulong offset = 0;
                    for (ulong i = 0; i < count; i++)
                    {
                        var feature = fc.Features[(int)index.Indices[i]];
                        var buffer = FeatureConversions.ToByteBuffer(feature, geometryType.Value, dimensions, columns);
                        featuresStream.Write(buffer, 0, buffer.Length);
                        offetsWriter.Write(offset);
                        offset += (ulong) buffer.Length;
                    }
                    var header = BuildHeader(count, offset, geometryType.Value, dimensions, columns, index);
                    memoryStream.Write(header, 0, header.Length);
                    featuresStream.WriteTo(memoryStream);
                    var indexBytes = index.ToBytes();
                    memoryStream.Write(indexBytes, 0, indexBytes.Length);
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
            
            var headerSize = (ulong) ByteBufferUtil.GetSizePrefix(bb);
            bb.Position = FlatBufferConstants.SizePrefixLength;
            var header = Header.GetRootAsHeader(bb);
            
            var count = header.FeaturesCount;
            var nodeSize = header.IndexNodeSize;

            bb.Position += (int) headerSize;

            var index = new PackedHilbertRTree(count, nodeSize);
            var indexData = bytes.Skip((int) (headerSize)).Take((int) index.Size).ToArray();
            index.Load(indexData);

            while (count-- > 0) {
                var featureLength = ByteBufferUtil.GetSizePrefix(bb);
                bb.Position += FlatBufferConstants.SizePrefixLength;
                var feature = FeatureConversions.FromByteBuffer(bb, header);
                fc.Add(feature);
                bb.Position += featureLength;
            }

            return fc;
        }

        private static byte[] BuildHeader(ulong count, ulong featuresSize, GeometryType geometryType, byte dimensions, IList<ColumnMeta> columns, PackedHilbertRTree index) {
            // TODO: size might not be enough, need to be adaptive
            var builder = new FlatBufferBuilder(1024);

            VectorOffset? columnsOffset = null;
            if (columns != null)
            {
                var columnsArray = columns
                .Select(c => Column.CreateColumn(builder, builder.CreateString(c.Name), c.Type))
                .ToArray();
                columnsOffset = Column.CreateSortedVectorOfColumn(builder, columnsArray);
            }

            Header.StartHeader(builder);
            Header.AddGeometryType(builder, geometryType);
            Header.AddDimensions(builder, dimensions);
            if (columnsOffset.HasValue)
                Header.AddColumns(builder, columnsOffset.Value);
            if (index != null)
                Header.AddIndexNodeSize(builder, 16);
            Header.AddFeaturesCount(builder, count);
            var offset = Header.EndHeader(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }
    }
}
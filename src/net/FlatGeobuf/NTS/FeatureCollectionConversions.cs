using System;
using System.Collections.Generic;
using System.Linq;
using System.IO;

using FlatBuffers;
using NetTopologySuite.Features;
using FlatGeobuf.Index;
using NetTopologySuite.Geometries;

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
        public static byte[] Serialize(FeatureCollection fc, GeometryType geometryType, byte dimensions = 2, IList<ColumnMeta> columns = null) {
            var featureFirst = fc.First();
            if (columns == null && featureFirst.Attributes != null)
                    columns = featureFirst.Attributes.GetNames()
                        .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
                        .ToList();
            using var memoryStream = new MemoryStream();
            Serialize(memoryStream, fc, geometryType, dimensions, columns);
            return memoryStream.ToArray();
        }

        public static void Serialize(Stream output, IEnumerable<IFeature> features, GeometryType geometryType, byte dimensions = 2, IList<ColumnMeta> columns = null) {
            output.Write(Constants.MagicBytes);
            var header = BuildHeader(0, geometryType, columns, null);
            output.Write(header);
            foreach (var feature in features)
            {
                var featureGeometryType = geometryType == GeometryType.Unknown ? GeometryConversions.ToGeometryType(feature.Geometry) : geometryType;
                var buffer = FeatureConversions.ToByteBuffer(feature, featureGeometryType, dimensions, columns);
                output.Write(buffer);
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

        public static FeatureCollection Deserialize(byte[] bytes, byte dimensions = 2) {
            var fc = new NetTopologySuite.Features.FeatureCollection();

            foreach (var feature in Deserialize(new MemoryStream(bytes), dimensions: dimensions))
                fc.Add(feature);

            return fc;
        }

        public static IEnumerable<IFeature> Deserialize(Stream stream, Envelope rect = null, byte dimensions = 2) {
            var reader = new BinaryReader(stream);
            var header = Helpers.ReadHeader(stream, out var headerSize);

            var count = header.FeaturesCount;
            var nodeSize = header.IndexNodeSize;
            var geometryType = header.GeometryType;

            IList<ColumnMeta> columns = null;
            if (header.ColumnsLength > 0)
            {
                columns = new List<ColumnMeta>();
                for (int i = 0; i < header.ColumnsLength; i++){
                    var column = header.Columns(i).Value;
                    columns.Add(new ColumnMeta() { Name = column.Name, Type = column.Type });
                }
            }

            if (nodeSize > 0)
            {
                long offset = 8 + 4 + headerSize;
                var size = PackedRTree.CalcSize(count, nodeSize);
                if (rect != null) {
                    var result = PackedRTree.StreamSearch(count, nodeSize, rect, (ulong treeOffset, ulong size) => {
                        stream.Seek(offset + (long) treeOffset, SeekOrigin.Begin);
                        return stream;
                    }).ToList();
                    foreach (var item in result) {
                        stream.Seek(offset + (long) size + (long) item.Offset, SeekOrigin.Begin);
                        var featureLength = reader.ReadInt32();
                        var feature = FeatureConversions.FromByteBuffer(new ByteBuffer(reader.ReadBytes(featureLength)), geometryType, dimensions, columns);
                        yield return feature;
                    }
                    yield break;
                }
                stream.Seek(8 + 4 + headerSize + (long) size, SeekOrigin.Begin);
            }

            while (stream.Position < stream.Length)
            {
                var featureLength = reader.ReadInt32();
                var feature = FeatureConversions.FromByteBuffer(new ByteBuffer(reader.ReadBytes(featureLength)), geometryType, dimensions, columns);
                yield return feature;
            }
        }

        private static byte[] BuildHeader(ulong count, GeometryType geometryType, IList<ColumnMeta> columns, PackedRTree index)
        {
            var builder = new FlatBufferBuilder(4096);

            VectorOffset? columnsOffset = null;
            if (columns != null)
            {
                var columnsArray = columns
                    .Select(c => Column.CreateColumn(builder, builder.CreateString(c.Name), c.Type))
                    .ToArray();
                columnsOffset = Header.CreateColumnsVector(builder, columnsArray);
            }

            Header.StartHeader(builder);
            Header.AddGeometryType(builder, geometryType);
            if (columnsOffset.HasValue)
                Header.AddColumns(builder, columnsOffset.Value);
            //if (index != null)
            Header.AddIndexNodeSize(builder, 0);
            Header.AddFeaturesCount(builder, count);
            var offset = Header.EndHeader(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }
    }
}
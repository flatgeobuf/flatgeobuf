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
        public static byte[] Serialize(FeatureCollection fc, GeometryType geometryType, byte dimensions = 2, IList<ColumnMeta> columns = null) {
            ulong count = (ulong) fc.Features.LongCount();

            if (count == 0)
                return new byte[0];
            
            /*var index = new PackedHilbertRTree(count);
            foreach (var f in fc.Features)
            {
                var b = f.Geometry.EnvelopeInternal;
                index.Add(b.MinX, b.MinY, b.MaxX, b.MaxY);
            }
            index.Finish();*/

            var featureFirst = fc.Features.First();

            if (columns == null && featureFirst.Attributes != null)
                columns = featureFirst.Attributes.GetNames()
                    .Select(n => new ColumnMeta() { Name = n, Type = ToColumnType(featureFirst.Attributes.GetType(n)) })
                    .ToList();

            using (var memoryStream = new MemoryStream())
            {
                memoryStream.Write(Constants.MagicBytes);

                using (var featuresStream = new MemoryStream())
                using (var offsetsStream = new MemoryStream())
                using (var offetsWriter = new BinaryWriter(offsetsStream))
                {
                    ulong offset = 0;
                    for (ulong i = 0; i < count; i++)
                    {
                        var feature = fc.Features[(int)i];
                        var featureGeometryType = geometryType == GeometryType.Unknown ? GeometryConversions.ToGeometryType(feature.Geometry) : geometryType;
                        var buffer = FeatureConversions.ToByteBuffer(feature, featureGeometryType, dimensions, columns);
                        featuresStream.Write(buffer, 0, buffer.Length);
                        offetsWriter.Write(offset);
                        offset += (ulong) buffer.Length;
                    }
                    var header = BuildHeader(count, geometryType, columns, null);
                    memoryStream.Write(header);
                    //var indexBytes = index.ToBytes();
                    //memoryStream.Write(indexBytes);
                    //offsetsStream.WriteTo(memoryStream);
                    featuresStream.WriteTo(memoryStream);
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

        public static FeatureCollection Deserialize(byte[] bytes) {
            var fc = new NetTopologySuite.Features.FeatureCollection();

            foreach (var feature in Deserialize(new MemoryStream(bytes)))
            {
                fc.Add(feature);
            }

            return fc;
        }

        public static IEnumerable<IFeature> Deserialize(Stream stream) {
            var reader = new BinaryReader(stream);

            var magicBytes = reader.ReadBytes(8);
            if (!magicBytes.SequenceEqual(Constants.MagicBytes))
                throw new Exception("Not a FlatGeobuf file");

            var headerSize = reader.ReadInt32();
            var header = Header.GetRootAsHeader(new ByteBuffer(reader.ReadBytes(headerSize)));
            
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
                var size = PackedHilbertRTree.CalcSize(count, nodeSize);
                stream.Seek((int) size, SeekOrigin.Current);
            }

            while (stream.Position < stream.Length)
            {
                var featureLength = reader.ReadInt32();
                var feature = FeatureConversions.FromByteBuffer(new ByteBuffer(reader.ReadBytes(featureLength)), geometryType, 2, columns);
                yield return feature;
            }
        }

        private static byte[] BuildHeader(ulong count, GeometryType geometryType, IList<ColumnMeta> columns, PackedHilbertRTree index)
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
using System;
using System.Linq;
using System.Collections.Generic;
using NetTopologySuite.Features;
using FlatBuffers;
using System.Text;
using System.IO;
using NTSGeometry = NetTopologySuite.Geometries.Geometry;

namespace FlatGeobuf.NTS
{
    public static class FeatureConversions
    {
        public static byte[] ToByteBuffer(IFeature feature, GeometryType geometryType, byte dimensions, IList<ColumnMeta> columns)
        {
            var builder = new FlatBufferBuilder(4096);
            var go = GeometryConversions.BuildGeometry(builder, feature.Geometry, geometryType, dimensions);
            var memoryStream = new MemoryStream();
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null)
            {
                var writer = new BinaryWriter(memoryStream, Encoding.UTF8);
                for (ushort i = 0; i < columns.Count(); i++)
                {
                    var column = columns[i];
                    var type = column.Type;
                    var name = column.Name;
                    if (!feature.Attributes.Exists(name))
                        continue;
                    var value = feature.Attributes[name];
                    if (value is null)
                        continue;
                    writer.Write(i);
                    switch (type)
                    {
                        case ColumnType.Bool:
                            writer.Write((bool) value);
                            break;
                        case ColumnType.Int:
                            writer.Write((int) value);
                            break;
                        case ColumnType.Long:
                            writer.Write((long) value);
                            break;
                        case ColumnType.Double:
                            writer.Write((double) value);
                            break;
                        case ColumnType.String:
                            var bytes = Encoding.UTF8.GetBytes((string) value);
                            writer.Write(bytes.Length);
                            writer.Write(bytes);
                            break;
                        default:
                            throw new ApplicationException("Unknown type " + value.GetType().FullName);
                    }
                }
            }

            var propertiesOffset = default(VectorOffset);
            if (memoryStream.Position > 0)
                propertiesOffset = Feature.CreatePropertiesVector(builder, memoryStream.ToArray());

            Offset<Geometry> geometryOffset;
            if (go.gos != null && go.gos.Length > 0) {
                var partOffsets = new Offset<Geometry>[go.gos.Length];
                for (int i = 0; i < go.gos.Length; i++) {
                    var goPart = go.gos[i];
                    var partOffset = Geometry.CreateGeometry(builder, goPart.endsOffset, goPart.coordsOffset, default, default, default, default, go.type, default);
                    partOffsets[i] = partOffset;
                }
                var partsOffset = Geometry.CreatePartsVector(builder, partOffsets);
                geometryOffset = Geometry.CreateGeometry(builder, default, default, default, default, default, default, go.type, partsOffset);
            } else {
                geometryOffset = Geometry.CreateGeometry(builder, go.endsOffset, go.coordsOffset, default, default, default, default, go.type, default);
            }
            Feature.StartFeature(builder);

            Feature.AddGeometry(builder, geometryOffset);
            Feature.AddProperties(builder, propertiesOffset);
            var featureOffset = Feature.EndFeature(builder);

            builder.FinishSizePrefixed(featureOffset.Value);

            return builder.DataBuffer.ToSizedArray();
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, GeometryType geometryType, byte dimensions, IList<ColumnMeta> columns = null)
        {
            var feature = Feature.GetRootAsFeature(bb);
            IAttributesTable attributesTable = null;
            var propertiesArray = feature.GetPropertiesArray();
            if (propertiesArray != null && propertiesArray.Length > 0)
            {
                var memoryStream = new MemoryStream(propertiesArray);
                var reader = new BinaryReader(memoryStream);
                attributesTable = new AttributesTable();
                while (memoryStream.Position < memoryStream.Length)
                {
                    ushort i = reader.ReadUInt16();
                    var column = columns[i];
                    var type = column.Type;
                    var name = column.Name;
                    switch (type)
                    {
                        case ColumnType.Bool:
                            attributesTable.Add(name, reader.ReadBoolean());
                            break;
                        case ColumnType.Short:
                            attributesTable.Add(name, reader.ReadInt16());
                            break;
                        case ColumnType.Int:
                            attributesTable.Add(name, reader.ReadInt32());
                            break;
                        case ColumnType.Long:
                            attributesTable.Add(name, reader.ReadInt64());
                            break;
                        case ColumnType.Double:
                            attributesTable.Add(name, reader.ReadDouble());
                            break;
                        case ColumnType.DateTime:
                        case ColumnType.String:
                            int len = reader.ReadInt32();
                            var str = Encoding.UTF8.GetString(memoryStream.ToArray(), (int) memoryStream.Position, len);
                            memoryStream.Position += len;
                            attributesTable.Add(name, str);
                            break;
                        default: throw new Exception($"Unknown type {type}");
                    }
                }
            }

            NTSGeometry geometry = null;
            try
            {
                if (feature.Geometry.HasValue)
                geometry = GeometryConversions.FromFlatbuf(feature.Geometry.Value, geometryType, dimensions);
            }
            catch (ArgumentException)
            {

            }

            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
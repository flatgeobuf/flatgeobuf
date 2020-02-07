using System;
using System.Linq;
using System.Collections.Generic;
using NetTopologySuite.Features;
using FlatBuffers;
using System.Text;
using System.IO;
using GeoAPI.Geometries;

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

            var geometryOffset = default(Offset<Geometry>);
            if (go.gos != null && go.gos.Length > 0) {
                var partOffsets = new Offset<Geometry>[go.gos.Length];
                for (int i = 0; i < go.gos.Length; i++) {
                    var goPart = go.gos[i];
                    var partOffset = Geometry.CreateGeometry(builder, goPart.endsOffset, goPart.coordsOffset, default(VectorOffset), default(VectorOffset), default(VectorOffset), default(VectorOffset), GeometryType.Unknown, default(VectorOffset));
                    partOffsets[i] = partOffset;
                }
                var partsOffset = Geometry.CreatePartsVector(builder, partOffsets);
                geometryOffset = Geometry.CreateGeometry(builder, default(VectorOffset), default(VectorOffset), default(VectorOffset), default(VectorOffset), default(VectorOffset), default(VectorOffset), GeometryType.Unknown, partsOffset);
            } else {
                geometryOffset = Geometry.CreateGeometry(builder, go.endsOffset, go.coordsOffset, default(VectorOffset), default(VectorOffset), default(VectorOffset), default(VectorOffset), GeometryType.Unknown, default(VectorOffset));
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
                            attributesTable.AddAttribute(name, reader.ReadBoolean());
                            break;
                        case ColumnType.Int:
                            attributesTable.AddAttribute(name, reader.ReadInt32());
                            break;
                        case ColumnType.Long:
                            attributesTable.AddAttribute(name, reader.ReadInt64());
                            break;
                        case ColumnType.Double:
                            attributesTable.AddAttribute(name, reader.ReadDouble());
                            break;
                        case ColumnType.String:
                            int len = reader.ReadInt32();
                            var str = Encoding.UTF8.GetString(memoryStream.ToArray(), (int) memoryStream.Position, len);
                            memoryStream.Position += len;
                            attributesTable.AddAttribute(name, str);
                            break;
                        default: throw new ApplicationException("Unknown type");
                    }
                }
            }

            IGeometry geometry = null;
            if (feature.Geometry.HasValue)
                geometry = GeometryConversions.FromFlatbuf(feature.Geometry.Value, geometryType);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
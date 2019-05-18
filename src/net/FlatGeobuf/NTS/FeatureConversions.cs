using System;
using System.Linq;
using System.Collections.Generic;
using NetTopologySuite.Features;
using FlatBuffers;

namespace FlatGeobuf.NTS
{
    public static class FeatureConversions
    {
        public static byte[] ToByteBuffer(IFeature feature, GeometryType geometryType, byte dimensions, IList<ColumnMeta> columns)
        {
            // TODO: size might not be enough, need to be adaptive
            var builder = new FlatBufferBuilder(1024);

            var go = GeometryConversions.BuildGeometry(builder, feature.Geometry, geometryType, dimensions);

            ByteBuffer bb = null;
            int offset = 0;
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null)
            {
                bb = new ByteBuffer(1024 * 1024);
                foreach (var column in columns)
                {
                    if (feature.Attributes.Exists(column.Name))
                    {
                        ushort columnIndex = (ushort)columns.IndexOf(column);
                        bb.PutUshort(offset, columnIndex);
                        offset += 2;
                        var value = feature.Attributes[column.Name];
                        switch (value)
                        {
                            case bool v:
                                bb.PutByte(offset, v ? (byte)1 : (byte)0);
                                offset += 1;
                                break;
                            case int v:
                                bb.PutInt(offset, v);
                                offset += 4;
                                break;
                            case long v:
                                bb.PutLong(offset, v);
                                offset += 8;
                                break;
                            case double v:
                                bb.PutDouble(offset, v);
                                offset += 8;
                                break;
                            case string v:
                                bb.PutInt(offset, v.Length);
                                offset += 4;
                                bb.PutStringUTF8(offset, v);
                                break;
                            case null:
                                break;
                            default: throw new ApplicationException("Unknown type");
                        }
                    }
                }
            }


            VectorOffset? propertiesOffset = null;
            if (bb != null)
                propertiesOffset = Feature.CreatePropertiesVector(builder, bb.ToArray(0, offset));

            Feature.StartFeature(builder);
            Feature.AddCoords(builder, go.coordsOffset.Value);
            if (go.lengthsOffset.HasValue)
                Feature.AddLengths(builder, go.lengthsOffset.Value);
            if (go.ringLengthsOffset.HasValue)
                Feature.AddRingLengths(builder, go.ringLengthsOffset.Value);
            if (go.ringCountsOffset.HasValue)
                Feature.AddRingCounts(builder, go.ringCountsOffset.Value);
            if (propertiesOffset.HasValue)
                Feature.AddProperties(builder, propertiesOffset.Value);
            var featureOffset = Feature.EndFeature(builder);

            builder.FinishSizePrefixed(featureOffset.Value);

            var bytes = builder.DataBuffer.ToSizedArray();

            return bytes;
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, Header header)
        {
            IList<Column> columns = null;
            if (header.ColumnsLength > 0)
            {
                columns = new List<Column>();
                for (int i = 0; i < header.ColumnsLength; i++)
                {
                    var column = header.Columns(i).Value;
                    columns.Add(column);
                }
            }

            var feature = Feature.GetRootAsFeature(bb);
            IAttributesTable attributesTable = null;

            var propertiesArray = feature.GetPropertiesArray();
            int offset = 0;
            if (propertiesArray != null && propertiesArray.Length > 0)
            {
                bb = new ByteBuffer(feature.GetPropertiesArray());
                attributesTable = new AttributesTable();
                while (offset < bb.Length)
                {
                    ushort i = bb.GetUshort(offset);
                    offset += 2;
                    var column = columns[i];
                    var type = column.Type;
                    var name = column.Name;
                    switch (type)
                    {
                        case ColumnType.Bool:
                            attributesTable.AddAttribute(name, bb.Get(offset) > 0 ? true : false);
                            offset += 1;
                            break;
                        case ColumnType.Int:
                            attributesTable.AddAttribute(name, bb.GetInt(offset));
                            offset += 4;
                            break;
                        case ColumnType.Long:
                            attributesTable.AddAttribute(name, bb.GetLong(offset));
                            offset += 8;
                            break;
                        case ColumnType.Double:
                            attributesTable.AddAttribute(name, bb.GetDouble(offset));
                            offset += 8;
                            break;
                        case ColumnType.String:
                            int len = bb.GetInt(offset);
                            offset += 4;
                            attributesTable.AddAttribute(name, bb.GetStringUTF8(offset, len));
                            offset += len;
                            break;
                        default: throw new ApplicationException("Unknown type");
                    }
                }
            }

            var geometry = GeometryConversions.FromFlatbuf(feature, header.GeometryType, header.Dimensions);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
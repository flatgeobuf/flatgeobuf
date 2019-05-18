using System;
using System.Linq;
using System.Collections.Generic;
using NetTopologySuite.Features;
using FlatBuffers;

namespace FlatGeobuf.NTS
{
    public static class FeatureConversions {
        public static byte[] ToByteBuffer(IFeature feature, GeometryType geometryType, byte dimensions, IList<ColumnMeta> columns) {
            // TODO: size might not be enough, need to be adaptive
            var builder = new FlatBufferBuilder(1024);

            var go = GeometryConversions.BuildGeometry(builder, feature.Geometry, geometryType, dimensions);

            
            /*VectorOffset? valuesOffset = null;
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null) {
                var valueOffsets = new List<Offset<Value>>();

                foreach (var column in columns) {
                    if (feature.Attributes.Exists(column.Name)) {
                        ushort columnIndex = (ushort) columns.IndexOf(column);
                        var value = feature.Attributes[column.Name];
                        switch(value) {
                            case bool v:
                                valueOffsets.Add(Value.CreateValue(builder, columnIndex, bool_value: v));
                                break;
                            case int v:
                                valueOffsets.Add(Value.CreateValue(builder, columnIndex, int_value: v));
                                break;
                            case long v:
                                valueOffsets.Add(Value.CreateValue(builder, columnIndex, long_value: v));
                                break;
                            case double v:
                                valueOffsets.Add(Value.CreateValue(builder, columnIndex, double_value: v));
                                break;
                            case string v:
                                valueOffsets.Add(Value.CreateValue(builder, columnIndex, string_valueOffset: builder.CreateString(v)));
                                break;
                            case null:
                                break;
                            default: throw new ApplicationException("Unknown type");
                        }
                    }
                }
                valuesOffset = Feature.CreateValuesVector(builder, valueOffsets.ToArray());
            }*/

            Feature.StartFeature(builder);
            Feature.AddCoords(builder, go.coordsOffset.Value);
            if (go.lengthsOffset.HasValue)
                Feature.AddLengths(builder, go.lengthsOffset.Value);
            if (go.ringLengthsOffset.HasValue)
                Feature.AddRingLengths(builder, go.ringLengthsOffset.Value);
            if (go.ringCountsOffset.HasValue)
                Feature.AddRingCounts(builder, go.ringCountsOffset.Value);
            
            //if (valuesOffset.HasValue)
            //    Feature.AddValues(builder, valuesOffset.Value);
            var offset = Feature.EndFeature(builder);

            builder.FinishSizePrefixed(offset.Value);

            var bytes = builder.DataBuffer.ToSizedArray();

            return bytes;
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, Header header) {
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

            //if (feature.ValuesLength > 0)
            //    attributesTable = new AttributesTable();

            /*for (int i = 0; i < feature.ValuesLength; i++)
            {
                var value = feature.Values(i).Value;
                var column = columns[value.ColumnIndex];
                switch (column.Type) {
                    case ColumnType.Bool:
                        attributesTable.AddAttribute(column.Name, value.BoolValue);
                        break;
                    case ColumnType.Int:
                        attributesTable.AddAttribute(column.Name, value.IntValue);
                        break;
                    case ColumnType.Long:
                        attributesTable.AddAttribute(column.Name, value.LongValue);
                        break;
                    case ColumnType.Double:
                        attributesTable.AddAttribute(column.Name, value.DoubleValue);
                        break;
                    case ColumnType.String:
                        attributesTable.AddAttribute(column.Name, value.StringValue);
                        break;
                    default: throw new ApplicationException("Unknown type");
                }
            }*/

            var geometry = GeometryConversions.FromFlatbuf(feature, header.GeometryType, header.Dimensions);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
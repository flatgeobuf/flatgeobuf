using System;
using System.Collections.Generic;
using NetTopologySuite.Features;
using FlatBuffers;

namespace FlatGeobuf.GeoJson
{
    public static class GeoJsonFeature {
        public static byte[] ToByteBuffer(IFeature feature, IList<ColumnMeta> columns) {
            // TODO: size might not be enough, need to be adaptive
            var builder = new FlatBufferBuilder(1024);

            var geometryOffset = GeoJsonGeometry.BuildGeometry(builder, feature.Geometry);
            
            VectorOffset? valuesOffset = null;
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
                            default: throw new ApplicationException("Unknown type");
                        }
                    }
                }
                valuesOffset = Feature.CreateValuesVector(builder, valueOffsets.ToArray());
            }

            Feature.StartFeature(builder);
            Feature.AddGeometry(builder, geometryOffset);
            if (valuesOffset.HasValue)
                Feature.AddValues(builder, valuesOffset.Value);
            var offset = Feature.EndFeature(builder);

            builder.FinishSizePrefixed(offset.Value);

            var bytes = builder.DataBuffer.ToSizedArray();

            return bytes;
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, Header header) {
            // TODO: introspect which layer
            var layer = header.Layers(0).Value;

            IList<Column> columns = null;
            if (layer.ColumnsLength > 0)
            {
                columns = new List<Column>();
                for (int i = 0; i < layer.ColumnsLength; i++)
                {
                    var column = layer.Columns(i).Value;
                    columns.Add(column);
                }
            }

            var feature = Feature.GetRootAsFeature(bb);
            IAttributesTable attributesTable = null;

            if (feature.ValuesLength > 0)
                attributesTable = new AttributesTable();

            for (int i = 0; i < feature.ValuesLength; i++)
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
            }

            var geometry = GeoJsonGeometry.FromFlatbuf(feature.Geometry.Value, layer.GeometryType, layer.Dimensions);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
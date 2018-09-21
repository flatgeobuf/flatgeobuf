using System;
using System.Collections.Generic;

using NetTopologySuite.IO;
using NetTopologySuite.Features;

using FlatBuffers;
using FlatGeobuf;

namespace FlatGeobuf.GeoJson
{
    public static class Feature {
        public static byte[] ToByteBuffer(IFeature feature, IList<ColumnMeta> columns) {
            var builder = new FlatBufferBuilder(1024);

            var geometryOffset = Geometry.BuildGeometry(builder, feature.Geometry);
            
            VectorOffset? valuesOffset = null;
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null) {
                var valueOffsets = new List<Offset<Value>>();

                foreach (var column in columns) {
                    if (feature.Attributes.Exists(column.Name)) {
                        ushort columnIndex = (ushort) columns.IndexOf(column);
                        var value = feature.Attributes[column.Name];
                        switch(value) {
                            case System.Int32 v:
                                valueOffsets.Add(FlatGeobuf.Value.CreateValue(builder, columnIndex, int_value: v));
                                break;
                            case System.Int64 v:
                                valueOffsets.Add(FlatGeobuf.Value.CreateValue(builder, columnIndex, long_value: v));
                                break;
                            case System.Double v:
                                valueOffsets.Add(FlatGeobuf.Value.CreateValue(builder, columnIndex, double_value: v));
                                break;
                            default: throw new ApplicationException("Unknown type");
                        }
                    }
                }
                valuesOffset = FlatGeobuf.Feature.CreateValuesVector(builder, valueOffsets.ToArray());
            }

            FlatGeobuf.Feature.StartFeature(builder);
            FlatGeobuf.Feature.AddGeometry(builder, geometryOffset);
            if (valuesOffset.HasValue)
                FlatGeobuf.Feature.AddValues(builder, valuesOffset.Value);
            var offset = FlatGeobuf.Feature.EndFeature(builder);

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

            var feature = FlatGeobuf.Feature.GetRootAsFeature(bb);
            IAttributesTable attributesTable = null;

            if (feature.ValuesLength > 0)
                attributesTable = new AttributesTable();

            for (int i = 0; i < feature.ValuesLength; i++)
            {
                var value = feature.Values(i).Value;
                var column = columns[value.ColumnIndex];
                switch (column.Type) {
                    case FlatGeobuf.ColumnType.Int:
                        attributesTable.AddAttribute(column.Name, value.IntValue);
                        break;
                    case FlatGeobuf.ColumnType.Long:
                        attributesTable.AddAttribute(column.Name, value.LongValue);
                        break;
                    case FlatGeobuf.ColumnType.Double:
                        attributesTable.AddAttribute(column.Name, value.DoubleValue);
                        break;
                }
            }

            var geometry = Geometry.FromFlatbuf(feature.Geometry.Value, layer.GeometryType, layer.Dimensions);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
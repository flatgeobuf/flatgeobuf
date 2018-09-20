using System;
using System.Collections.Generic;

using NetTopologySuite.IO;
using NetTopologySuite.Features;

using FlatBuffers;
using FlatGeobuf;

namespace FlatGeobuf.GeoJson
{
    public static class Feature {
        public static byte[] ToByteBuffer(IFeature feature, Dictionary<string, ColumnType> columns) {
            var builder = new FlatBufferBuilder(1024);

            var geometryOffset = Geometry.BuildGeometry(builder, feature.Geometry);

            //VectorOffset? valuesLengthsOffset = null;
            VectorOffset? valuesOffset = null;
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null) {                
                var byteBuffer = new FlatBuffers.ByteBuffer(1024);
                
                var offset2 = 0;
                foreach (var key in columns.Keys) {
                    if (feature.Attributes.Exists(key)) {
                        var value = feature.Attributes[key];
                        switch(value) {
                            case System.Int32 v: {
                                byteBuffer.PutInt(offset2, v);
                                offset2 += 4;
                                break;
                            }
                            case System.Int64 v: {
                                byteBuffer.PutLong(offset2, v);
                                offset2 += 8;
                                break;
                            }
                            case System.Double v: {
                                byteBuffer.PutDouble(offset2, v);
                                offset2 += 8;
                                break;
                            }
                            default: throw new ApplicationException("Unknown type");
                        }
                    }
                }
                valuesOffset = FlatGeobuf.Feature.CreateValuesVector(builder, byteBuffer.ToFullArray());
            }

            FlatGeobuf.Feature.StartFeature(builder);
            FlatGeobuf.Feature.AddGeometry(builder, geometryOffset);
            if (valuesOffset.HasValue)
                FlatGeobuf.Feature.AddValues(builder, valuesOffset.Value);
            var offset = FlatGeobuf.Feature.EndFeature(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, IDictionary<string, ColumnType> columns) {
            var feature = FlatGeobuf.Feature.GetRootAsFeature(bb);
            var valuesArray = feature.GetValuesArray();
            IAttributesTable attributesTable = null;
            if (valuesArray != null) {
                attributesTable = new AttributesTable();
                var byteBuffer = new ByteBuffer(valuesArray);

                var offset = 0;
                foreach (var column in columns)
                {
                    switch (column.Value) {
                        case FlatGeobuf.ColumnType.INT: {
                            var v = byteBuffer.GetInt(offset);
                            attributesTable.AddAttribute(column.Key, v);
                            offset += 4;
                            break;
                        }
                        case FlatGeobuf.ColumnType.LONG: {
                            var v = byteBuffer.GetLong(offset);
                            attributesTable.AddAttribute(column.Key, v);
                            offset += 8;
                            break;
                        }
                        case FlatGeobuf.ColumnType.DOUBLE: {
                            var v = byteBuffer.GetDouble(offset);
                            attributesTable.AddAttribute(column.Key, v);
                            offset += 8;
                            break;
                        }
                    }
                }
            }

            var geometry = Geometry.FromFlatbuf(feature.Geometry.Value);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
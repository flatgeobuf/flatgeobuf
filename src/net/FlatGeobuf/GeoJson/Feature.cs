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
            var builder = new FlatBufferBuilder(40);

            var geometryOffset = Geometry.BuildGeometry(builder, feature.Geometry);

            //VectorOffset? valuesLengthsOffset = null;
            VectorOffset? valuesOffset = null;
            if (feature.Attributes != null && feature.Attributes.Count > 0 && columns != null) {                
                var byteByffer = new FlatBuffers.ByteBuffer(40);
                foreach (var key in columns.Keys) {
                    if (feature.Attributes.Exists(key)) {
                        var value = feature.Attributes[key];
                        byteByffer.PutDouble(0, (double) value);
                    }
                }
                FlatGeobuf.Feature.CreateValuesVector(builder, byteByffer.ToFullArray());
            }

            FlatGeobuf.Feature.StartFeature(builder);
            FlatGeobuf.Feature.AddGeometry(builder, geometryOffset);
            if (valuesOffset.HasValue)
                FlatGeobuf.Feature.AddValues(builder, valuesOffset.Value);
            var offset = FlatGeobuf.Feature.EndFeature(builder);

            builder.FinishSizePrefixed(offset.Value);

            return builder.DataBuffer.ToSizedArray();
        }

        public static IFeature FromByteBuffer(ByteBuffer bb, IDictionary<string, ColumnType> column) {
            var feature = FlatGeobuf.Feature.GetRootAsFeature(bb);
            var valuesArray = feature.GetValuesArray();
            IAttributesTable attributesTable = null;
            if (valuesArray != null) {
                attributesTable = new AttributesTable();
                var byteBuffer = new ByteBuffer(valuesArray);
                var value = byteBuffer.GetDouble(0);
                attributesTable.AddAttribute("test", 1.1);
            }

            var geometry = Geometry.FromFlatbuf(feature.Geometry.Value);
            var f = new NetTopologySuite.Features.Feature(geometry, attributesTable);
            return f;
        }
    }
}
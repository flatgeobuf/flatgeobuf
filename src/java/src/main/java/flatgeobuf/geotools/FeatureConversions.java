package flatgeobuf.geotools;

import java.io.IOException;

import com.google.flatbuffers.FlatBufferBuilder;

import flatgeobuf.generated.*;

import org.geotools.feature.simple.SimpleFeatureBuilder;
import org.opengis.feature.simple.SimpleFeature;

public class FeatureConversions {

    public static byte[] serialize(SimpleFeature feature, long fid, int geometryType, int dimensions) throws IOException {
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);
        org.locationtech.jts.geom.Geometry geometry = (org.locationtech.jts.geom.Geometry) feature.getDefaultGeometry();

        
        int valuesOffset = 0;
        // TODO: parse values
        /*for (int j = 0; j < types.size(); j++) {
            Object value = simpleFeature.getAttribute(j);
            AttributeDescriptor ad = types.get(j);
            if (id_option != null && id_option.equals(ad.getLocalName())) {
                continue; // skip this value as it is used as the id
            }
            if (ad instanceof GeometryDescriptor) {
                // multiple geometries per feature is not supported
            } else {
                //key = ad.getLocalName();
                //value;
            }
        }*/
        GeometryOffsets go = GeometryConversions.serialize(builder, geometry, geometryType, dimensions);
        int featureOffset = Feature.createFeature(builder, fid, go.ringCountsOffset, go.ringLengthsOffset, go.lengthsOffset, go.coordsOffset, valuesOffset);
        builder.finishSizePrefixed(featureOffset);

        return builder.sizedByteArray();
    }

    public static SimpleFeature deserialize(Feature feature, SimpleFeatureBuilder fb, int geometryType, int dimensions) {
        long fid = feature.fid();
        fb.add(GeometryConversions.deserialize(feature, geometryType, dimensions));
        SimpleFeature f = fb.buildFeature(Long.toString(fid));
        return f;
    }
}
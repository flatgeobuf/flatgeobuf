package flatgeobuf.geotools;

import java.io.IOException;
import java.io.OutputStream;
import java.util.*;

import com.google.flatbuffers.FlatBufferBuilder;

import flatgeobuf.generated.*;

import org.geotools.data.simple.SimpleFeatureCollection;
import org.geotools.feature.FeatureIterator;
import org.locationtech.jts.geom.Geometry;
import org.locationtech.jts.geom.Point;
import org.opengis.feature.simple.SimpleFeature;
import org.opengis.feature.simple.SimpleFeatureType;
import org.opengis.feature.type.AttributeDescriptor;
import org.opengis.feature.type.GeometryDescriptor;

public class FeatureCollectionConversions {

    public class ColumnMeta {
        public String name;
        public ColumnType type;
    }

    public class HeaderMeta {
        public String name;
        public GeometryType geometryType;
        public byte dimensions;
        public List<ColumnMeta> columns;
    }

    public static void write(SimpleFeatureCollection featureCollection,
            OutputStream outputStream) throws IOException {

        byte[] header = BuildHeader(featureCollection.getSchema());
        outputStream.write(header);

        try (FeatureIterator<SimpleFeature> iterator = featureCollection.features()) {
            SimpleFeatureType fType;
            List<AttributeDescriptor> types;
            while (iterator.hasNext()) {
                SimpleFeature simpleFeature = iterator.next();
                fType = simpleFeature.getFeatureType();
                types = fType.getAttributeDescriptors();
                
                /*
                // write the simple feature id
                if (id_option == null) {
                    // no specific attribute nominated, use the simple feature id
                    jsonWriter.key("id").value(simpleFeature.getID());
                } else if (id_option.length() != 0) {
                    // a specific attribute was nominated to be used as id
                    Object value = simpleFeature.getAttribute(id_option);
                    jsonWriter.key("id").value(value);
                }
                */

                // set that axis order that should be used to write geometries
                /*GeometryDescriptor defaultGeomType = fType.getGeometryDescriptor();
                if (defaultGeomType != null) {
                    CoordinateReferenceSystem featureCrs =
                            defaultGeomType.getCoordinateReferenceSystem();
                    jsonWriter.setAxisOrder(CRS.getAxisOrder(featureCrs));
                    if (crs == null) {
                        crs = featureCrs;
                    }
                } else {
                    // If we don't know, assume EAST_NORTH so that no swapping occurs
                    jsonWriter.setAxisOrder(CRS.AxisOrder.EAST_NORTH);
                }*/

                // start writing the simple feature geometry JSON object
                Geometry aGeom = (Geometry) simpleFeature.getDefaultGeometry();
                
                // start writing feature properties JSON object
                for (int j = 0; j < types.size(); j++) {
                    Object value = simpleFeature.getAttribute(j);
                    AttributeDescriptor ad = types.get(j);
                    /*if (id_option != null && id_option.equals(ad.getLocalName())) {
                        continue; // skip this value as it is used as the id
                    }*/
                    if (ad instanceof GeometryDescriptor) {
                        // multiple geometries per feature is not supported
                    } else {
                        //jsonWriter.key(ad.getLocalName());
                        //jsonWriter.value(value);
                    }
                }
                // Bounding box for feature in properties
                //ReferencedEnvelope refenv =
                //        ReferencedEnvelope.reference(simpleFeature.getBounds());
                /*if (featureBounding && !refenv.isEmpty()) {
                    //jsonWriter.writeBoundingBox(refenv);
                }*/
                //jsonWriter.endObject(); // end the properties

                //writeExtraFeatureProperties(simpleFeature, operation, jsonWriter);

                //jsonWriter.endObject(); // end the feature
            }
        }
    }

    private static byte toGeometryType(Class<?> geometryClass) {
        if (geometryClass.isAssignableFrom(Point.class))
            return GeometryType.Point;
        else
            throw new RuntimeException("Unknown geometry type");
    }

    private static byte[] BuildHeader(SimpleFeatureType simpleFeatureType) {
        // TODO: size might not be enough, need to be adaptive
        FlatBufferBuilder builder = new FlatBufferBuilder(1024);

        byte geometryType = toGeometryType(simpleFeatureType.getGeometryDescriptor().getType().getBinding());

        Header.startHeader(builder);
        Header.addGeometryType(builder, geometryType);
        int offset = Header.endHeader(builder);

        builder.finishSizePrefixed(offset);

        return builder.sizedByteArray();
    }
}
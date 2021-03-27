package org.wololo.flatgeobuf.geotools;

import org.locationtech.jts.geom.Envelope;
import org.opengis.feature.simple.SimpleFeatureType;

import java.util.List;

public class HeaderMeta {
    public String name;
    public byte geometryType;
    public Envelope envelope;
    public long featuresCount;
    public boolean hasZ = false;
    public boolean hasM = false;
    public boolean hasT = false;
    public boolean hasTM = false;
    public int indexNodeSize;
    public List<ColumnMeta> columns;
    public SimpleFeatureType featureType;
    public int offset;
}
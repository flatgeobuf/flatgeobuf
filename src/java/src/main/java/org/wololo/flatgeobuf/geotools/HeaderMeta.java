package org.wololo.flatgeobuf.geotools;

import java.util.List;

public class HeaderMeta {
    public String name;
    public byte geometryType;
    public long featuresCount;
    public boolean hasZ = false;
    public boolean hasM = false;
    public boolean hasT = false;
    public List<ColumnMeta> columns;
}
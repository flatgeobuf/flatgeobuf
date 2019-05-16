package flatgeobuf.geotools;

import java.util.List;

public class HeaderMeta {
    public String name;
    public byte geometryType;
    public byte dimensions;
    public List<ColumnMeta> columns;
}
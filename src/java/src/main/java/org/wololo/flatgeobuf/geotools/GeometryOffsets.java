package org.wololo.flatgeobuf.geotools;

public class GeometryOffsets {
    int coordsOffset;
    public int[] ends = null;
    public int[] lengths = null;
    public int endsOffset = 0;
    public int lengthsOffset = 0;
    public GeometryOffsets[] gos = null;
}
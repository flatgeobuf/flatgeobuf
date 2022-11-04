package org.wololo.flatgeobuf;

public class GeometryOffsets {
    public int coordsOffset;
    public long[] ends = null;
    public int[] lengths = null;
    public int endsOffset = 0;
    public int lengthsOffset = 0;
    public int type = 0;
    public GeometryOffsets[] gos = null;
}
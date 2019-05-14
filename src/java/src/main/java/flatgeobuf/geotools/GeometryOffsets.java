package flatgeobuf.geotools;

public class GeometryOffsets {
    int coordsOffset;
    public int[] lengths = null;
    public int[] ringLengths = null;
    public int[] ringCounts = null;
    public int lengthsOffset = 0;
    public int ringLengthsOffset = 0;
    public int ringCountsOffset = 0;
}
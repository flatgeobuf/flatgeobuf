using NetTopologySuite.Geometries;

namespace FlatGeobuf.NTS
{
    public class FlatGeobufCoordinateSequence : CoordinateSequence
    {
        private readonly Geometry _geometry;
        private readonly int _offset;

        public FlatGeobufCoordinateSequence(ref Header header, ref Geometry geometry, int end = 0)
            : base(GetCount(ref geometry, end), GetDimension(ref header), GetMeasures(ref header))
        {
            _geometry = geometry;
            _offset = end > 0 ? (int) geometry.Ends(end - 1) : 0;
        }

        static int GetCount(ref Geometry geometry, int end)
        {
            if (geometry.EndsLength == 0)
                return geometry.XyLength / 2;
            else if (end > 0)
                return (int) geometry.Ends(end) - (int) geometry.Ends(end - 1);
            else
                return (int) geometry.Ends(0);
        }

        static int GetDimension(ref Header header)
        {
            var dimension = 2;
            if (header.HasZ)
                dimension++;
            if (header.HasM)
                dimension++;
            return dimension;
        }

        static int GetMeasures(ref Header header)
        {
            if (header.HasM)
                return 1;
            else
                return 0;
        }

        public override CoordinateSequence Copy()
        {
            return null;
        }

        public override double GetX(int index)
        {
            return _geometry.Xy((_offset + index) * 2);
        }

        public override double GetY(int index)
        {
            return _geometry.Xy((_offset + index) * 2 + 1);
        }

        public override double GetZ(int index)
        {
            return _geometry.Z(_offset + index);
        }

        public override double GetM(int index)
        {
            return _geometry.M(_offset + index);
        }

        public override double GetOrdinate(int index, int ordinateIndex)
        {
            if (ordinateIndex == 0)
                return GetX(index);
            else if (ordinateIndex == 1)
                return GetY(index);
            else if (ordinateIndex == 2)
                return GetZ(index);
            else if (ordinateIndex == 3)
                return GetM(index);
            return Coordinate.NullOrdinate;
        }

        public override void SetOrdinate(int index, int ordinateIndex, double value)
        {
            if (ordinateIndex == 0)
                _geometry.GetXyBytes()[(_offset + index) * 2] = value;
            else if (ordinateIndex == 1)
                _geometry.GetXyBytes()[(_offset + index) * 2 + 1] = value;
            else if (ordinateIndex == 2)
                _geometry.GetZArray()[_offset + index] = value;
            else if (ordinateIndex == 3)
                _geometry.GetMArray()[_offset + index] = value;
        }
    }
}

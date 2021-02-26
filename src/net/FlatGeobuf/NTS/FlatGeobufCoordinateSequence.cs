using NetTopologySuite.Geometries;

namespace FlatGeobuf.NTS
{
    public class FlatGeobufCoordinateSequence : CoordinateSequence
    {
        private readonly int _offset;

        private readonly double[] _xy;
        private readonly double[] _z;
        private readonly double[] _m;

        public double[] XY { get { return _xy; } }
        public double[] Z { get { return _z; } }
        public double[] M { get { return _m; } }

        public FlatGeobufCoordinateSequence(double[] xy, double[] z, double[] m, int count, int offset)
            : base(count, GetDimension(z, m), m != null ? 1 : 0)
        {
            _offset = offset;
            _xy = xy;
            _z = z;
            _m = m;
        }

        static int GetDimension(double[] z, double[] m)
        {
            var dimension = 2;
            if (z != null)
                dimension++;
            if (m != null)
                dimension++;
            return dimension;
        }

        public override CoordinateSequence Copy()
        {
            return null;
        }

        public override double GetX(int index)
        {
            return _xy[(_offset + index) * 2];
        }

        public override double GetY(int index)
        {
            return _xy[(_offset + index) * 2 + 1];
        }

        public override double GetZ(int index)
        {
            return _z[_offset + index];
        }

        public override double GetM(int index)
        {
            return _m[_offset + index];
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
                _xy[(_offset + index) * 2] = value;
            else if (ordinateIndex == 1)
                _xy[(_offset + index) * 2 + 1] = value;
            else if (ordinateIndex == 2)
                _z[_offset + index] = value;
            else if (ordinateIndex == 3)
                _m[_offset + index] = value;
        }
    }
}

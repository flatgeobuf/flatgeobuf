using NetTopologySuite.Geometries;

namespace FlatGeobuf.NTS
{
    public class FlatGeobufGeometryCoordinateSequence : CoordinateSequence
    {
        private readonly Geometry _geometry;
        private readonly int _offset;

        public FlatGeobufGeometryCoordinateSequence(int count, int dimension, ref Geometry geometry, int offset = 0)
            : base(count, dimension, 0)
        {
            _geometry = geometry;
            _offset = offset;
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
            return;
        }
    }
}

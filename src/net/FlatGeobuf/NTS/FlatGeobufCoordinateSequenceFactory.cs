using System;
using System.Collections.Generic;
using FlatGeobuf;
using FlatGeobuf.NTS;
using NetTopologySuite.Geometries;
using NetTopologySuite.Geometries.Implementation;

namespace FlatGeobuf.NTS
{
    public sealed class FlatGeobufCoordinateSequenceFactory : CoordinateSequenceFactory
    {
        public CoordinateSequence Create(ref Header header, ref Geometry geometry, int offset = 0)
        {
            return new FlatGeobufCoordinateSequence(ref header, ref geometry, offset);
        }

        public override CoordinateSequence Create(int size, int dimension, int measures)
        {
            throw new NotImplementedException();
        }
    }
}
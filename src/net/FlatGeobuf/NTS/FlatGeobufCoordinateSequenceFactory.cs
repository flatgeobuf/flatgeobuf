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
        public CoordinateSequence Create(HeaderT header, ref Geometry geometry, int end = 0)
        {
            var xy = geometry.GetXyArray();
            var offset = end > 0 ? (int) geometry.Ends(end - 1) : 0;
            var count = geometry.EndsLength > 0 ? (int) geometry.Ends(end) - offset : xy.Length / 2;
            double[] z = null;
            double[] m = null;
            if (header.HasZ)
                z = geometry.GetZArray();
            if (header.HasM)
                m = geometry.GetMArray();
            return new FlatGeobufCoordinateSequence(xy, z, m, count, offset);
        }

        public override CoordinateSequence Create(int size, int dimension, int measures)
        {
            double[] xy = new double[size * 2];
            double[] z = null;
            double[] m = null;
            return new FlatGeobufCoordinateSequence(xy, z, m, size, 0);
        }
    }
}
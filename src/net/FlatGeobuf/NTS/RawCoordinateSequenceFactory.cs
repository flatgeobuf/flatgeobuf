using System;
using System.Collections.Generic;

namespace NetTopologySuite.Geometries.Implementation
{
    /// <summary>
    /// Factory for creating <see cref="RawCoordinateSequence"/> instances.
    /// </summary>
    public sealed class RawCoordinateSequenceFactory : CoordinateSequenceFactory
    {
        private readonly Ordinates _ordinatesInGroups;

        private readonly Ordinates[] _ordinateGroups;

        /// <summary>
        /// Initializes a new instance of the <see cref="RawCoordinateSequenceFactory"/> class.
        /// </summary>
        /// <param name="ordinateGroups">
        /// A sequence of zero or more <see cref="Ordinates"/> flags representing ordinate values
        /// that should be allocated together.
        /// </param>
        /// <exception cref="ArgumentNullException">
        /// Thrown when <paramref name="ordinateGroups"/> is <see langword="null"/>.
        /// </exception>
        /// <exception cref="ArgumentException">
        /// Thrown when a given flag appears in more than one element of
        /// <paramref name="ordinateGroups"/>.
        /// </exception>
        /// <remarks>
        /// Any flags not represented in <paramref name="ordinateGroups"/>, and any spatial or
        /// measure dimensions beyond the 16th, will be allocated together, SoA-style.
        /// <para/>
        /// Elements without any bits set will be silently ignored.
        /// </remarks>
        public RawCoordinateSequenceFactory(IEnumerable<Ordinates> ordinateGroups)
        {
            if (ordinateGroups is null)
            {
                throw new ArgumentNullException(nameof(ordinateGroups));
            }

            var seenOrdinates = Ordinates.None;
            var ordinateGroupsList = new List<Ordinates>();
            foreach (var ordinateGroup in ordinateGroups)
            {
                if ((ordinateGroup & seenOrdinates) != Ordinates.None)
                {
                    throw new ArgumentException("Each ordinate may show up in at most one group.", nameof(ordinateGroups));
                }

                seenOrdinates |= ordinateGroup;

                if (OrdinatesUtility.OrdinatesToDimension(ordinateGroup) < 2)
                {
                    // it would have been equally correct to omit this
                    continue;
                }

                _ordinatesInGroups |= ordinateGroup;
                ordinateGroupsList.Add(ordinateGroup);
            }

            _ordinateGroups = ordinateGroupsList.ToArray();
        }

        /// <inheritdoc />
        public override CoordinateSequence Create(int size, int dimension, int measures)
        {
            int spatial = dimension - measures;
            var ordinatesInGroups = _ordinatesInGroups;
            var ordinatesInResult = Ordinates.None;
            double[] underlyingData = new double[size * dimension];
            var rawDataList = new List<Memory<double>>(dimension);
            var remainingRawData = underlyingData.AsMemory();
            var dimensionMap = new (int RawDataIndex, int DimensionIndex)[dimension];

            for (int i = 0; i < spatial; i++)
            {
                if (i <= 16)
                {
                    var flag = (Ordinates)((int)Ordinates.Spatial1 << i);
                    ordinatesInResult |= flag;
                    if ((ordinatesInGroups & flag) != Ordinates.None)
                    {
                        continue;
                    }
                }

                dimensionMap[i].RawDataIndex = rawDataList.Count;
                rawDataList.Add(remainingRawData.Slice(0, size));
                remainingRawData = remainingRawData.Slice(size);
            }

            for (int i = 0; i < measures; i++)
            {
                if (i <= 16)
                {
                    var flag = (Ordinates)((int)Ordinates.Measure1 << i);
                    ordinatesInResult |= flag;
                    if ((ordinatesInGroups & flag) != Ordinates.None)
                    {
                        continue;
                    }
                }

                dimensionMap[spatial + i].RawDataIndex = rawDataList.Count;
                rawDataList.Add(remainingRawData.Slice(0, size));
                remainingRawData = remainingRawData.Slice(size);
            }

            if ((ordinatesInResult & ordinatesInGroups) == Ordinates.None)
            {
                return new RawCoordinateSequence(rawDataList.ToArray(), dimensionMap, measures);
            }

            foreach (var overallOrdinateGroup in _ordinateGroups)
            {
                var ordinateGroup = overallOrdinateGroup & ordinatesInResult;
                if (ordinateGroup == Ordinates.None)
                {
                    continue;
                }

                int dimCountForGroup = 0;
                for (int i = 0; i < spatial && i < 16; i++)
                {
                    if ((ordinateGroup & (Ordinates)((int)Ordinates.Spatial1 << i)) == Ordinates.None)
                    {
                        continue;
                    }

                    dimensionMap[i].RawDataIndex = rawDataList.Count;
                    dimensionMap[i].DimensionIndex = dimCountForGroup++;
                }

                for (int i = 0; i < measures && i < 16; i++)
                {
                    if ((ordinateGroup & (Ordinates)((int)Ordinates.Measure1 << i)) == Ordinates.None)
                    {
                        continue;
                    }

                    dimensionMap[spatial + i].RawDataIndex = rawDataList.Count;
                    dimensionMap[spatial + i].DimensionIndex = dimCountForGroup++;
                }

                rawDataList.Add(remainingRawData.Slice(0, size * dimCountForGroup));
                remainingRawData = remainingRawData.Slice(size * dimCountForGroup);
            }

            return new RawCoordinateSequence(rawDataList.ToArray(), dimensionMap, measures);
        }
    }
}
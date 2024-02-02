using FlatGeobuf.Index;
using Google.FlatBuffers;
using NetTopologySuite;
using NetTopologySuite.Features;
using NetTopologySuite.Geometries;
using System;
using System.Buffers;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;

namespace FlatGeobuf.NTS
{
    /// <summary>
    /// Async enumerator class to access <see cref="IFeature"/> from a <see cref="Stream"/> in an asnychronous way.
    /// </summary>
    public class AsyncFeatureEnumerator : IAsyncEnumerator<IFeature>
    {
        private static readonly FlatGeobufCoordinateSequenceFactory CsFactory =
            new FlatGeobufCoordinateSequenceFactory();

        private readonly GeometryFactory _factory;
        private readonly Stream _stream;
        private readonly HeaderT _header;
        private readonly long _dataOffset;
        private readonly CancellationToken _token;

        // Index
        private readonly HashSet<long> _itemsIndex;
        private readonly IEnumerator<(long Offset, ulong Index)> _itemEnumerator;

        public static async Task<AsyncFeatureEnumerator> Create(Stream stream, PrecisionModel pm = null, Envelope rect = null, CancellationToken? token = null)
        {
            // Ensure stream is not null
            if (stream == null)
                throw new ArgumentNullException(nameof(stream));

            // Test if stream is readable
            if (!stream.CanRead)
                throw new ArgumentException(nameof(stream));

            // Ensure stream is at start
            if (stream.Position > 0)
                throw new ArgumentException("Not at start of stream", nameof(stream));

            // Ensure token is not null
            token ??= CancellationToken.None;

            // Read the header
            var header = await Helpers.ReadHeaderAsync(stream, token.Value);

            // Create an appropriate geometry factory
            pm ??= NtsGeometryServices.Instance.DefaultPrecisionModel;
            var factory = NtsGeometryServices.Instance.CreateGeometryFactory(pm, header.Crs.Code, CsFactory);

            // Get filter iterator
            rect ??= new Envelope();
            IList<(long Offset, ulong Index)> filter = null;
            if (header.IndexNodeSize > 0)
            {
                if (rect.IsNull)
                {
                    await SkipIndexAsync(header, stream, token.Value);
                }
                else
                {
                    filter = await ReadIndexAsync(header, stream, rect, token.Value);
                }
            }

            return new AsyncFeatureEnumerator(factory, header, stream, filter, token.Value);
        }

        /// <summary>
        /// Creates an instance of this class
        /// </summary>
        /// <param name="factory">The geometry factory to use</param>
        /// <param name="header">The header, containg general information about the feature data set</param>
        /// <param name="stream">The stream from which to deserialize the feature data set</param>
        /// <param name="items">An object containing the interesting features</param>
        private AsyncFeatureEnumerator(GeometryFactory factory, HeaderT header, Stream stream, IList<(long Offset, ulong Index)> items, CancellationToken token)
        {
            _factory = factory;
            _header = header;
            _stream = stream;
            _dataOffset = stream.Position;
            _token = token;

            // Build the items index
            if (items != null)
            {
                if (stream.CanSeek)
                {
                    _itemEnumerator = items.GetEnumerator(); ;
                }
                else
                {
                    _itemsIndex = CreateItemsIndex(items);
                }
            }
        }

        /// <summary>
        /// Gets a value indicating the title of the provided feature data set
        /// </summary>
        public string Title { get => _header.Title; }

        /// <summary>
        /// Gets a value indicating the number of features in the provided data set
        /// </summary>
        public int NumFeatures { get => (int)_header.FeaturesCount; }

        /// <summary>
        /// Gets a value indicating the extent of the provided data set
        /// </summary>
        public Envelope Extent { get => new Envelope(_header.Envelope[0], _header.Envelope[2], _header.Envelope[1], _header.Envelope[3]); }

        /// <summary>
        /// Gets a value indicating the spatial reference id of the provided feature data set
        /// </summary>
        public int SRID { get => _header.Crs.Code; }

        /// <summary>
        /// Gets a value indicating the <c>Coordinate Reference System</c> of the provided feature data set
        /// </summary>
        public CrsT Crs { get => _header.Crs; }

        /// <inheritdoc/>
        public IFeature Current { get; private set; }

        /// <inheritdoc/>
        #pragma warning disable CS1998
        public async ValueTask DisposeAsync()
        {
            _itemEnumerator?.Dispose();
#if NETSTANDARD2_1
            await _stream.DisposeAsync();
#else
            _stream.Dispose();
#endif
        }
        #pragma warning restore CS1998

        /// <inheritdoc/>
        public async ValueTask<bool> MoveNextAsync()
        {
            // Initialize current
            Current = null;

            // If we have an index via enumerator position the stream accordingly
            if (_itemEnumerator != null)
            {
                // If there are no more items left, return false
                if (!_itemEnumerator.MoveNext())
                    return false;

                _stream.Seek(_dataOffset + _itemEnumerator.Current.Offset, SeekOrigin.Begin);
            }

            // If we are at the end of the stream, there is no more data coming
            else if (_stream.Position >= _stream.Length)
            {
                return false;
            }

            // Get the current position
            long position = _stream.Position;

            // Read the feature size
            byte[] smallBuffer = new byte[4];
            int numRead = await _stream.ReadAsync(smallBuffer, 0, 4, _token);
            if (numRead != 4) throw new InvalidDataException("Insufficient stream length");
            int featureSize = MemoryMarshal.Read<int>(smallBuffer);

            // provide buffer, read feature data
            byte[] featureData = ArrayPool<byte>.Shared.Rent(featureSize);
            numRead = await _stream.ReadAsync(featureData, 0, featureSize, _token);
            if (numRead != featureSize) throw new InvalidDataException("Insufficient stream length");

            // Check if the this feature is requested
            if (_itemsIndex != null && !_itemsIndex.Contains(position))
                return await MoveNextAsync();

            // Create the feature
            Current = FeatureConversions.FromByteBuffer(_factory, CsFactory, new ByteBuffer(featureData, 0), _header);

            // free buffer 
            ArrayPool<byte>.Shared.Return(featureData);

            // return success
            return true;
        }

        #region static utility methods


        private static async ValueTask<IList<(long Offset, ulong Index)>>
            ReadIndexAsync(HeaderT header, Stream stream, Envelope rect, CancellationToken token)
        {
            int treeSize = (int)PackedRTree.CalcSize(header.FeaturesCount, header.IndexNodeSize);
            var dataOffset = stream.Position + treeSize;
            List<(long Offset, ulong Index)> filter;
            if (stream.CanSeek)
            {
                filter = PackedRTree.StreamSearch(stream, header.FeaturesCount, header.IndexNodeSize, rect);
                stream.Seek(dataOffset, SeekOrigin.Begin);
            }
            else
            {
                byte[] treeData = ArrayPool<byte>.Shared.Rent(treeSize);
                int numRead = await stream.ReadAsync(treeData, 0, treeSize, token);
                if (numRead != treeSize) throw new InvalidDataException("Insufficient stream size");

                // Read the spatial index
                filter = PackedRTree.StreamSearch(new MemoryStream(treeData, 0, treeSize), header.FeaturesCount, header.IndexNodeSize, rect);
                ArrayPool<byte>.Shared.Return(treeData);
            }
            return filter;
        }

        private static async ValueTask SkipIndexAsync(HeaderT header, Stream stream, CancellationToken token)
        {
            int treeSize = (int)PackedRTree.CalcSize(header.FeaturesCount, header.IndexNodeSize);
            if (stream.CanSeek)
            {
                stream.Seek(treeSize, SeekOrigin.Current);
            }
            else
            {
                byte[] treeData = ArrayPool<byte>.Shared.Rent(treeSize);
                int numRead = await stream.ReadAsync(treeData, 0, treeSize, token);
                if (numRead != treeSize) throw new InvalidDataException("Insufficient stream size");
                ArrayPool<byte>.Shared.Return(treeData);
            }

        }
        private static HashSet<long> CreateItemsIndex(IEnumerable<(long Offset, ulong Index)> items)
        {
            var res = new HashSet<long>();
            foreach ((long offset, _) in items)
                res.Add(offset);
            return res;
        }

        #endregion
    }
}

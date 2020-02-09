using System;
using System.Collections.Generic;
using System.IO;

namespace FlatGeobuf.Index
{
    struct NodeItem {
        double minX;
        double minY;
        double maxX;
        double maxY;
        ulong offset;
    };

    struct Rect
    {
        public double minX;
        public double minY;
        public double maxX;
        public double maxY;

        public double Width => maxX - minX;
        public double Height => maxY - minY;

        public Rect(double minX, double minY, double maxX, double maxY)
        {
            this.minX = minX;
            this.minY = minY;
            this.maxX = maxX;
            this.maxY = maxY;
        }

        public void Expand(Rect r)
        {
            if (r.minX < minX) minX = r.minX;
            if (r.minY < minY) minY = r.minY;
            if (r.maxX > maxX) maxX = r.maxX;
            if (r.maxY > maxY) maxY = r.maxY;
        }
        
        public bool Intersects(Rect r)
        {
            if (maxX < r.minX) return false;
            if (maxY < r.minY) return false;
            if (minX > r.maxX) return false;
            if (minY > r.maxY) return false;
            return true;
        }

        public static Rect CreateInvertedInfiniteRect()
        {
            var r = new Rect();
            r.minX = Double.PositiveInfinity;
            r.minY = Double.PositiveInfinity;
            r.maxX = Double.NegativeInfinity;
            r.maxY = Double.NegativeInfinity;
            return r;
        }
    }

    /// <summary>
    /// Packed Hilbert R-Tree implementation
    /// 
    /// Based on https://github.com/mourner/flatbush
    /// </summary>
    public class PackedHilbertRTree
    {
        Rect _extent = Rect.CreateInvertedInfiniteRect();
        Rect[] _rects;
        ulong[] _indices;
        ulong _pos;

        ulong _numItems;
        ulong _numNodes;
        ushort _nodeSize;

        IList<ulong> _levelBounds;

        public ulong NumNodes => _numNodes;

        public ulong Size => _numNodes * 4 * 8 + _numNodes * 8;

        public ulong[] Indices => _indices;

        public PackedHilbertRTree(ulong numItems, ushort nodeSize = 16)
        {
            _numItems = numItems;
            _nodeSize = Math.Min(Math.Max(nodeSize, (ushort) 2), (ushort) 65535);

            var n = numItems;
            var numNodes = n;
            _levelBounds = new List<ulong>() { n };
            do
            {
                n = (ulong) Math.Ceiling((double) n / _nodeSize);
                numNodes += n;
                _levelBounds.Add(numNodes);
            } while (n != 1);

            _numNodes = numNodes;

            _rects = new Rect[_numNodes];
            _indices = new ulong[_numNodes];
        }

        public void Load(byte[] data)
        {
            FromBytes(data, _numNodes);
        }

        public void Add(double minX, double minY, double maxX, double maxY)
        {
            var r = new Rect(minX, minY, maxX, maxY);
            _indices[_pos] = _pos;
            _rects[_pos] = r;
            _extent.Expand(r);
            _pos++;
        }

        public void Finish()
        {
            var hilbertMax = (1 << 16) - 1;

            // map item centers into Hilbert coordinate space and calculate Hilbert values
            ulong[] hilbertValues = new ulong[_numItems];
            for (ulong i = 0; i < _numItems; i++)
            {
                var r = _rects[i];
                var x = (ulong) Math.Floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.Width);
                var y = (ulong) Math.Floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.Height);
                hilbertValues[i] = Hilbert(x, y);
            }

            // sort items by their Hilbert value (for packing later)
            Sort(hilbertValues, _rects, _indices, 0, _numItems - 1);
            
            // generate nodes at each tree level, bottom-up
            for (ushort i = 0, pos = 0; i < _levelBounds.Count - 1; i++)
            {
                var end = _levelBounds[i];

                while (pos < end)
                {
                    var nodeRect = Rect.CreateInvertedInfiniteRect();
                    var nodeIndex = pos;
                    for (var j = 0; j < _nodeSize && pos < end; j++)
                        nodeRect.Expand(_rects[pos++]);
                    _rects[_pos] = nodeRect;
                    _indices[_pos] = nodeIndex;
                    _pos++;
                }
            }
        }
        

        public IList<ulong> Search(double minX, double minY, double maxX, double maxY)
        {
            var r = new Rect(minX, minY, maxX, maxY);

            var nodeIndex = (ulong) _rects.LongLength - 1UL;
            ushort level = (ushort) (_levelBounds.Count - 1);
            Stack<ulong> stack = new Stack<ulong>();
            IList<ulong> results = new List<ulong>();
            
            while(true)
            {
                // find the end index of the node
                var end = Math.Min(nodeIndex + _nodeSize, _levelBounds[level]);

                // search through child nodes
                for (var pos = nodeIndex; pos < end; pos++)
                {
                    ulong index = _indices[pos];

                    // check if node bbox intersects with query bbox
                    if (!r.Intersects(_rects[pos])) continue;

                    if (nodeIndex < _numItems)
                    {
                        results.Add(index); // leaf item
                    }
                    else
                    {
                        stack.Push(index); // node; add it to the search queue
                        stack.Push(level - 1UL);
                    }
                }

                if (stack.Count == 0)
                    break;
                level = (ushort) stack.Pop();
                nodeIndex = stack.Pop();
            }

            return results;
        }

        // custom quicksort that sorts bbox data alongside the hilbert values
        static void Sort(ulong[] values, Rect[] boxes, ulong[] indices, ulong left, ulong right)
        {
            if (left >= right) return;

            var pivot = values[(left + right) >> 1];
            var i = left - 1;
            var j = right + 1;

            while (true)
            {
                do i++; while (values[i] < pivot);
                do j--; while (values[j] > pivot);
                if (i >= j) break;
                Swap(values, boxes, indices, i, j);
            }

            Sort(values, boxes, indices, left, j);
            Sort(values, boxes, indices, j + 1, right);
        }

        // swap two values and two corresponding boxes
        static void Swap(ulong[] values, Rect[] boxes, ulong[] indices, ulong i, ulong j)
        {
            var temp = values[i];
            values[i] = values[j];
            values[j] = temp;
            
            var r = boxes[i];
            boxes[i] = boxes[j];
            boxes[j] = r;
            
            var e = indices[i];
            indices[i] = indices[j];
            indices[j] = e;
        }

        // Fast Hilbert curve algorithm by http://threadlocalmutex.com/
        // Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
        static ulong Hilbert(ulong x, ulong y)
        {
            var a = x ^ y;
            var b = 0xFFFF ^ a;
            var c = 0xFFFF ^ (x | y);
            var d = x & (y ^ 0xFFFF);

            var A = a | (b >> 1);
            var B = (a >> 1) ^ a;
            var C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
            var D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

            a = A; b = B; c = C; d = D;
            A = ((a & (a >> 2)) ^ (b & (b >> 2)));
            B = ((a & (b >> 2)) ^ (b & ((a ^ b) >> 2)));
            C ^= ((a & (c >> 2)) ^ (b & (d >> 2)));
            D ^= ((b & (c >> 2)) ^ ((a ^ b) & (d >> 2)));

            a = A; b = B; c = C; d = D;
            A = ((a & (a >> 4)) ^ (b & (b >> 4)));
            B = ((a & (b >> 4)) ^ (b & ((a ^ b) >> 4)));
            C ^= ((a & (c >> 4)) ^ (b & (d >> 4)));
            D ^= ((b & (c >> 4)) ^ ((a ^ b) & (d >> 4)));

            a = A; b = B; c = C; d = D;
            C ^= ((a & (c >> 8)) ^ (b & (d >> 8)));
            D ^= ((b & (c >> 8)) ^ ((a ^ b) & (d >> 8)));

            a = C ^ (C >> 1);
            b = D ^ (D >> 1);

            var i0 = x ^ y;
            var i1 = b | (0xFFFF ^ (i0 | a));

            i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
            i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
            i0 = (i0 | (i0 << 2)) & 0x33333333;
            i0 = (i0 | (i0 << 1)) & 0x55555555;

            i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
            i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
            i1 = (i1 | (i1 << 2)) & 0x33333333;
            i1 = (i1 | (i1 << 1)) & 0x55555555;

            return ((i1 << 1) | i0) >> 0;
        }

        public byte[] ToBytes()
        {
            using (var stream = new MemoryStream())
            using (var writer = new BinaryWriter(stream))
            {
                foreach (var r in _rects)
                {
                    writer.Write(r.minX);
                    writer.Write(r.minY);
                    writer.Write(r.maxX);
                    writer.Write(r.maxY);
                }
                foreach (var i in _indices)
                {
                    writer.Write(i);
                }
                return stream.ToArray();
            }
        }
        
        void FromBytes(byte[] data, ulong numNodes)
        {
            using (var stream = new MemoryStream(data))
            using (var reader = new BinaryReader(stream))
            {
                for (ulong i = 0; i < numNodes; i++)
                {
                    _rects[i].minX = reader.ReadDouble();
                    _rects[i].minY = reader.ReadDouble();
                    _rects[i].maxX = reader.ReadDouble();
                    _rects[i].maxY = reader.ReadDouble();
                }

                for (ulong i = 0; i < numNodes; i++)
                {
                    _indices[i] = reader.ReadUInt64();
                }
            }
        }

        public static ulong CalcSize(ulong numItems, ushort nodeSize)
        {
            if (nodeSize < 2)
                throw new Exception("Node size must be at least 2");
            if (numItems == 0)
                throw new Exception("Number of items must be greater than 0");
            ushort nodeSizeMin = Math.Min(Math.Max(nodeSize, (ushort) 2), (ushort) 65535);
            // limit so that resulting size in bytes can be represented by uint64_t
            if (numItems > 1 << 56)
                throw new OverflowException("Number of items must be less than 2^56");
            ulong n = numItems;
            ulong numNodes = n;
            do {
                n = (n + nodeSizeMin - 1) / nodeSizeMin;
                numNodes += n;
            } while (n != 1);
            return numNodes * (8 * 5);
        }
    }
}

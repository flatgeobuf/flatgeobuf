using System;
using System.Linq;
using System.Collections.Generic;

namespace FlatGeobuf.Index
{
    struct Rect
    {
        public double minX;
        public double minY;
        public double maxX;
        public double maxY;
        public int hilbertValue;

        public double Width => maxX - minX;
        public double Height => maxY - minY;

        public Rect(double minX, double minY, double maxX, double maxY)
        {
            this.minX = minX;
            this.minY = minY;
            this.maxX = maxX;
            this.maxY = maxY;
            hilbertValue = -1;
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
        IList<Rect> _rects;

        int _numItems;
        int _nodeSize;

        IList<int> _levelBounds;

        public PackedHilbertRTree(int numItems, int nodeSize = 16)
        {
            _numItems = numItems;
            _nodeSize = Math.Min(Math.Max(nodeSize, 2), 65535);

            var n = numItems;
            var numNodes = n;
            _levelBounds = new List<int>() { n };
            do
            {
                n = (int) Math.Ceiling((double) n / _nodeSize);
                numNodes += n;
                _levelBounds.Add(numNodes);
            } while (n != 1);

            _rects = new List<Rect>(numNodes);
        }

        public void Add(double minX, double minY, double maxX, double maxY)
        {
            var r = new Rect(minX, minY, maxX, maxY);
            _rects.Add(r);
            _extent.Expand(r);
        }

        public void Finish()
        {
            var hilbertMax = (1 << 16) - 1;

            // map item centers into Hilbert coordinate space and calculate Hilbert values
            for (var i = 0; i < _rects.Count; i++)
            {
                var r = _rects[i];
                var x = (int) Math.Floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.Width);
                var y = (int) Math.Floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.Height);
                r.hilbertValue = CalcHilbertValue(x, y);
            }

            // sort items by their Hilbert value (for packing later)
            _rects = _rects.OrderBy(r => r.hilbertValue).ToList();
            
            // generate nodes at each tree level, bottom-up
            for (int i = 0, pos = 0; i < _levelBounds.Count - 1; i++)
            {
                var end = _levelBounds[i];
                
                while (pos < end)
                {
                    var nodeRect = Rect.CreateInvertedInfiniteRect();
                    var nodeIndex = pos;
                    for (var j = 0; j < _nodeSize && pos < end; j++)
                        nodeRect.Expand(_rects[pos++ + j]);
                    _rects.Add(nodeRect);
                }
            }
        }

        public IList<int> Search(double minX, double minY, double maxX, double maxY)
        {
            var r = new Rect(minX, minY, maxX, maxY);

            int nodeIndex = _rects.Count - 1;
            var level = _levelBounds.Count - 1;
            Queue<int> queue = new Queue<int>();
            IList<int> results = new List<int>();
            
            while(true)
            {
                // find the end index of the node
                var end = Math.Min(nodeIndex + _nodeSize, _levelBounds[level]);

                // search through child nodes
                for (var pos = nodeIndex; pos < end; pos++)
                {
                    // check if node bbox intersects with query bbox
                    if (!r.Intersects(_rects[pos])) continue;

                    if (nodeIndex < _numItems)
                    {
                        results.Add(pos); // leaf item
                    }
                    else
                    {
                        queue.Enqueue(pos); // node; add it to the search queue
                        queue.Enqueue(level - 1);
                    }
                }

                if (queue.Count == 0)
                    break;
                level = queue.Dequeue();
                nodeIndex = queue.Dequeue();
            }

            return results;
        }

        // Fast Hilbert curve algorithm by http://threadlocalmutex.com/
        // Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
        static int CalcHilbertValue(int x, int y)
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
    }
}

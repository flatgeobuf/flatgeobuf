using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using GeoAPI.Geometries;

namespace FlatGeobuf.Index
{
    public class PackedRTree
    {
        private const ulong NODE_ITEM_LEN = 8 * 4 + 8;

        public delegate Stream ReadNode(ulong offset, ulong length);

        double MinX { get; set; } = double.MinValue;
        double MinY { get; set; } = double.MinValue;
        double MaxX { get; set; } = double.MaxValue;
        double MaxY { get; set; } = double.MaxValue;

        ulong NumItems { get; set; }
        ulong NumNodes { get; set; }
        ushort NodeSize { get; set; }

        IList<(ulong Start, ulong End)> LevelBounds { get; set; }

        private byte[] _data;
        private ulong _pos;

        public PackedRTree(ulong numItems, ushort nodeSize) {
            if (nodeSize < 2)
                throw new ArgumentException("Node size must be at least 2");
            NodeSize = nodeSize;
            if (numItems == 0)
                throw new ArgumentException("Cannot create empty tree");
            NumItems = numItems;
            NodeSize = Math.Min(Math.Max(nodeSize, (ushort) 2), (ushort) 65535);
            LevelBounds = GenerateLevelBounds(NumItems, NodeSize);
            NumNodes = LevelBounds.First().End;
            _data = new byte[NumNodes * NODE_ITEM_LEN];
        }

        private void GenerateNodes()
        {
            for (int i = 0; i < LevelBounds.Count - 1; i++) {
                var pos = LevelBounds[i].Start;
                var readerStream = new MemoryStream(_data);
                readerStream.Position = (long) pos;
                var reader = new BinaryReader(readerStream);
                var end = LevelBounds[i].End;
                var newpos = LevelBounds[i + 1].Start;
                var writerStream = new MemoryStream(_data);
                writerStream.Position = (long) newpos;
                var writer = new BinaryWriter(writerStream);
                while (pos < end) {
                    var nodeMinX = double.MinValue;
                    var nodeMinY = double.MinValue;
                    var nodeMaxX = double.MaxValue;
                    var nodeMaxY = double.MaxValue;
                    for (var j = 0; j < NodeSize && pos < end; j++) {
                        var minX = reader.ReadDouble();
                        var minY = reader.ReadDouble();
                        var maxX = reader.ReadDouble();
                        var maxY = reader.ReadDouble();
                        reader.ReadUInt64();
                        if (MinX < nodeMinX) nodeMinX = minX;
                        if (MinY < nodeMinY) nodeMinY = minY;
                        if (MaxX > nodeMaxX) nodeMaxX = maxX;
                        if (MaxY > nodeMaxY) nodeMaxY = maxY;
                    }
                    writer.Write(nodeMinX);
                    writer.Write(nodeMinY);
                    writer.Write(nodeMaxX);
                    writer.Write(nodeMaxY);
                    writer.Write(pos);
                }
            }
        }

        private static uint hilbert(uint x, uint y)
        {
            uint a = x ^ y;
            uint b = 0xFFFF ^ a;
            uint c = 0xFFFF ^ (x | y);
            uint d = x & (y ^ 0xFFFF);

            uint A = a | (b >> 1);
            uint B = (a >> 1) ^ a;
            uint C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
            uint D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

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

            uint i0 = x ^ y;
            uint i1 = b | (0xFFFF ^ (i0 | a));

            i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
            i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
            i0 = (i0 | (i0 << 2)) & 0x33333333;
            i0 = (i0 | (i0 << 1)) & 0x55555555;

            i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
            i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
            i1 = (i1 | (i1 << 2)) & 0x33333333;
            i1 = (i1 | (i1 << 1)) & 0x55555555;

            uint value = ((i1 << 1) | i0);

            return value;
        }

        public static ulong CalcSize(ulong numItems, ushort nodeSize)
        {
            if (nodeSize < 2)
                throw new Exception("Node size must be at least 2");
            if (numItems == 0)
                throw new Exception("Number of items must be greater than 0");
            ushort nodeSizeMin = Math.Min(Math.Max(nodeSize, (ushort) 2), (ushort) 65535);
            // limit so that resulting size in bytes can be represented by ulong
            if (numItems > 1 << 56)
                throw new OverflowException("Number of items must be less than 2^56");
            ulong n = numItems;
            ulong numNodes = n;
            do {
                n = (n + nodeSizeMin - 1) / nodeSizeMin;
                numNodes += n;
            } while (n != 1);
            return numNodes * NODE_ITEM_LEN;
        }

        static IList<(ulong Start, ulong End)> GenerateLevelBounds(ulong numItems, ushort nodeSize) {
            if (nodeSize < 2)
                throw new Exception("Node size must be at least 2");
            if (numItems == 0)
                throw new Exception("Number of items must be greater than 0");
            
            // number of nodes per level in bottom-up order
            var n = numItems;
            var numNodes = n;
            var levelNumNodes = new List<ulong>() { n };
            do {
                n = (n + nodeSize - 1) / nodeSize;
                numNodes += n;
                levelNumNodes.Add(n);
            } while (n != 1);

            // bounds per level in reversed storage order (top-down)
            var levelOffsets = new List<ulong>();
            n = numNodes;
            foreach (var size in levelNumNodes) {
                levelOffsets.Add(n - size);
                n -= size;
            };
            levelOffsets.Reverse();
            levelNumNodes.Reverse();
            var levelBounds = new List<(ulong Start, ulong End)>();
            for (var i = 0; i < levelNumNodes.Count; i++)
                levelBounds.Add((levelOffsets[i], levelOffsets[i] + levelNumNodes[i]));
            levelBounds.Reverse();
            return levelBounds;
        }

        public static IEnumerable<(ulong Offset, ulong Index)> StreamSearch(ulong numItems, ushort nodeSize, Envelope rect, ReadNode readNode)
        {
            var minX = rect.MinX;
            var minY = rect.MinY;
            var maxX = rect.MaxX;
            var maxY = rect.MaxY;
            var levelBounds = GenerateLevelBounds(numItems, nodeSize);
            var numNodes = levelBounds.First().End;
            var stack = new Stack<(ulong NodeIndex, int Level)>();
            stack.Push((0UL, levelBounds.Count() - 1));
            while (stack.Count != 0) {
                var (nodeIndex, level) = stack.Pop();
                var isLeafNode = nodeIndex >= numNodes - numItems;
                // find the end index of the node
                var levelBound = levelBounds[level].End;
                var end = Math.Min(nodeIndex + nodeSize, levelBound);
                var length = end - nodeIndex;
                var stream = readNode(nodeIndex * NODE_ITEM_LEN, length * NODE_ITEM_LEN);
                var start = stream.Position;
                var reader = new BinaryReader(stream);
                // search through child nodes
                for (var pos = nodeIndex; pos < end; pos++) {
                    stream.Seek(start + (long) ((pos - nodeIndex) * NODE_ITEM_LEN), SeekOrigin.Begin);
                    if (maxX < reader.ReadDouble()) continue; // maxX < nodeMinX
                    if (maxY < reader.ReadDouble()) continue; // maxY < nodeMinY
                    if (minX > reader.ReadDouble()) continue; // minX > nodeMaxX
                    if (minY > reader.ReadDouble()) continue; // minY > nodeMaxY
                    var offset = reader.ReadUInt64();
                    if (isLeafNode)
                        yield return (offset, pos - 1);
                    else
                        stack.Push((offset, level - 1));
                }
                // order queue to traverse sequential
                //queue.sort((a, b) => b[0] - a[0])
            }
        }
    }
}

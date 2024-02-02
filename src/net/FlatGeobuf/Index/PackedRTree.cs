using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using NetTopologySuite.Geometries;

namespace FlatGeobuf.Index
{
    public class PackedRTree
    {
        private const ulong NODE_ITEM_LEN = 8 * 4 + 8;

        public delegate Stream ReadNode(ulong offset, ulong length);

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
            var levelBounds = new List<(ulong Start, ulong End)>();
            for (var i = 0; i < levelNumNodes.Count; i++)
                levelBounds.Add((levelOffsets[i], levelOffsets[i] + levelNumNodes[i]));
            return levelBounds;
        }

        internal static List<(long Offset, ulong Index)> StreamSearch(Stream stream, ulong numItems, ushort nodeSize, Envelope rect)
        {
            var treePosition = stream.Position;
            var minX = rect.MinX;
            var minY = rect.MinY;
            var maxX = rect.MaxX;
            var maxY = rect.MaxY;
            var levelBounds = GenerateLevelBounds(numItems, nodeSize);
            var leafNodesOffset = levelBounds.First().Start;
            var numNodes = levelBounds.First().End;
            var stack = new Stack<(ulong NodeIndex, int Level)>();
            stack.Push((0UL, levelBounds.Count() - 1));
            using var reader = new BinaryReader(stream, Encoding.UTF8, true);
            var res = new List<(long Offset, ulong Index)>((int)numItems);
            while (stack.Count != 0)
            {
                var (nodeIndex, level) = stack.Pop();
                var isLeafNode = nodeIndex >= numNodes - numItems;
                // find the end index of the node
                var levelBound = levelBounds[level].End;
                var end = Math.Min(nodeIndex + nodeSize, levelBound);
                stream.Seek(treePosition + (long)(nodeIndex * NODE_ITEM_LEN), SeekOrigin.Begin);
                var start = (long)(nodeIndex * NODE_ITEM_LEN);
                // search through child nodes
                for (var pos = nodeIndex; pos < end; pos++)
                {
                    stream.Seek(treePosition + start + (long)((pos - nodeIndex) * NODE_ITEM_LEN), SeekOrigin.Begin);
                    if (maxX < reader.ReadDouble()) continue; // maxX < nodeMinX
                    if (maxY < reader.ReadDouble()) continue; // maxY < nodeMinY
                    if (minX > reader.ReadDouble()) continue; // minX > nodeMaxX
                    if (minY > reader.ReadDouble()) continue; // minY > nodeMaxY
                    var offset = reader.ReadUInt64();
                    if (isLeafNode)
                        res.Add(((long)offset, pos - leafNodesOffset));
                    else
                        stack.Push((offset, level - 1));
                }
                // order queue to traverse sequential
                //queue.sort((a, b) => b[0] - a[0])
            }
            return res;
        }

        public static IEnumerable<(ulong Offset, ulong Index)> StreamSearch(ulong numItems, ushort nodeSize, Envelope rect, ReadNode readNode)
        {
            var minX = rect.MinX;
            var minY = rect.MinY;
            var maxX = rect.MaxX;
            var maxY = rect.MaxY;
            var levelBounds = GenerateLevelBounds(numItems, nodeSize);
            var leafNodesOffset = levelBounds.First().Start;
            var numNodes = levelBounds.First().End;
            var stack = new Stack<(ulong NodeIndex, int Level)>();
            stack.Push((0UL, levelBounds.Count() - 1));
            while (stack.Count != 0)
            {
                var (nodeIndex, level) = stack.Pop();
                var isLeafNode = nodeIndex >= numNodes - numItems;
                // find the end index of the node
                var levelBound = levelBounds[level].End;
                var end = Math.Min(nodeIndex + nodeSize, levelBound);
                var length = end - nodeIndex;
                var stream = readNode(nodeIndex * NODE_ITEM_LEN, length * NODE_ITEM_LEN);
                var start = stream.Position;
                using var reader = new BinaryReader(stream, Encoding.UTF8, true);
                // search through child nodes
                for (var pos = nodeIndex; pos < end; pos++)
                {
                    stream.Seek(start + (long)((pos - nodeIndex) * NODE_ITEM_LEN), SeekOrigin.Begin);
                    if (maxX < reader.ReadDouble()) continue; // maxX < nodeMinX
                    if (maxY < reader.ReadDouble()) continue; // maxY < nodeMinY
                    if (minX > reader.ReadDouble()) continue; // minX > nodeMaxX
                    if (minY > reader.ReadDouble()) continue; // minY > nodeMaxY
                    var offset = reader.ReadUInt64();
                    if (isLeafNode)
                        yield return (offset, pos - leafNodesOffset);
                    else
                        stack.Push((offset, level - 1));
                }
                // order queue to traverse sequential
                //queue.sort((a, b) => b[0] - a[0])
            }
        }
    }
}

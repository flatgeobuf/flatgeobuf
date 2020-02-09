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

    

    public class PackedHilbertRTree
    {
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

package org.wololo.flatgeobuf;

import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Stack;

import org.locationtech.jts.geom.Envelope;

public class PackedRTree {
    private static int NODE_ITEM_LEN = 8 * 4 + 8;

    public static long calcSize(int numItems, int nodeSize) {
        if (nodeSize < 2)
            throw new RuntimeException("Node size must be at least 2");
        if (numItems == 0)
            throw new RuntimeException("Number of items must be greater than 0");
        int nodeSizeMin = Math.min(Math.max(nodeSize, 2), 65535);
        // limit so that resulting size in bytes can be represented by ulong
        if (numItems > 1 << 56)
            throw new IndexOutOfBoundsException("Number of items must be less than 2^56");
        int n = numItems;
        int numNodes = n;
        do {
            n = (n + nodeSizeMin - 1) / nodeSizeMin;
            numNodes += n;
        } while (n != 1);
        return numNodes * NODE_ITEM_LEN;
    }

    static ArrayList<Integer> generateLevelEnds(int numItems, int nodeSize) {
        if (nodeSize < 2)
            throw new RuntimeException("Node size must be at least 2");
        if (numItems == 0)
            throw new RuntimeException("Number of items must be greater than 0");

        // number of nodes per level in bottom-up order
        int n = numItems;
        int numNodes = n;
        ArrayList<Integer> levelNumNodes = new ArrayList<Integer>();
        levelNumNodes.add(n);
        do {
            n = (n + nodeSize - 1) / nodeSize;
            numNodes += n;
            levelNumNodes.add(n);
        } while (n != 1);

        // offsets per level in reversed storage order (top-down)
        ArrayList<Integer> levelOffsets = new ArrayList<Integer>();
        n = numNodes;
        for (int size : levelNumNodes) {
            levelOffsets.add(n - size);
            n -= size;
        }
        ArrayList<Integer> levelEnds = new ArrayList<Integer>();
        for (int i = 0; i < levelNumNodes.size(); i++)
            levelEnds.add(levelOffsets.get(i) + levelNumNodes.get(i));
        return levelEnds;
    }

    private static class StackItem {
        public StackItem(long nodeIndex, int level) {
            this.nodeIndex = nodeIndex;
            this.level = level;
        }

        long nodeIndex;
        int level;
    }

    public static class SearchHit {
        public SearchHit(long offset, long index) {
            this.offset = offset;
            this.index = index;
        }

        public long offset;
        public long index;
    }

    public static ArrayList<SearchHit> search(ByteBuffer bb, int start, int numItems, int nodeSize, Envelope rect) {
        ArrayList<SearchHit> searchHits = new ArrayList<SearchHit>();
        double minX = rect.getMinX();
        double minY = rect.getMinY();
        double maxX = rect.getMaxX();
        double maxY = rect.getMaxY();
        ArrayList<Integer> levelEnds = generateLevelEnds(numItems, nodeSize);
        int numNodes = levelEnds.get(0);
        Stack<StackItem> stack = new Stack<StackItem>();
        stack.add(new StackItem(0, levelEnds.size() - 1));
        while (stack.size() != 0) {
            StackItem stackItem = stack.pop();
            int nodeIndex = (int) stackItem.nodeIndex;
            int level = stackItem.level;
            boolean isLeafNode = nodeIndex >= numNodes - numItems;
            // find the end index of the node
            int levelEnd = levelEnds.get(level);
            int end = Math.min(nodeIndex + nodeSize, levelEnd);
            int nodeStart = start + (nodeIndex * NODE_ITEM_LEN);
            // int length = end - nodeIndex;
            // search through child nodes
            for (int pos = nodeIndex; pos < end; pos++) {
                int offset = nodeStart + ((pos - nodeIndex) * NODE_ITEM_LEN);
                double nodeMinX = bb.getDouble(offset + 0);
                double nodeMinY = bb.getDouble(offset + 8);
                double nodeMaxX = bb.getDouble(offset + 16);
                double nodeMaxY = bb.getDouble(offset + 24);
                if (maxX < nodeMinX)
                    continue;
                if (maxY < nodeMinY)
                    continue;
                if (minX > nodeMaxX)
                    continue;
                if (minY > nodeMaxY)
                    continue;
                long indexOffset = bb.getLong(offset + 32);
                if (isLeafNode)
                    searchHits.add(new SearchHit(indexOffset, pos - 1));
                else
                    stack.add(new StackItem(indexOffset, level - 1));
            }
        }
        return searchHits;
    }

    public static class SearchResult {
        public ArrayList<SearchHit> hits = new ArrayList<SearchHit>();
        public int pos;
    }

    public static SearchResult search(
            InputStream stream, int start, int numItems, int nodeSize, Envelope rect)
            throws IOException {
        LittleEndianDataInputStream data = new LittleEndianDataInputStream(stream);
        int dataPos = 0;
        int skip;
        SearchResult searchResult = new SearchResult();
        double minX = rect.getMinX();
        double minY = rect.getMinY();
        double maxX = rect.getMaxX();
        double maxY = rect.getMaxY();
        ArrayList<Integer> levelEnds = generateLevelEnds(numItems, nodeSize);
        int numNodes = levelEnds.get(0);
        Stack<StackItem> stack = new Stack<StackItem>();
        stack.add(new StackItem(0, levelEnds.size() - 1));
        while (stack.size() != 0) {
            StackItem stackItem = stack.pop();
            int nodeIndex = (int) stackItem.nodeIndex;
            int level = stackItem.level;
            boolean isLeafNode = nodeIndex >= numNodes - numItems;
            // find the end index of the node
            int levelBound = levelEnds.get(level);
            int end = Math.min(nodeIndex + nodeSize, levelBound);
            int nodeStart = nodeIndex * NODE_ITEM_LEN;
            skip = nodeStart - dataPos;
            if (skip > 0) {
                skipNBytes(data, skip);
                dataPos += skip;
            }
            // int length = end - nodeIndex;
            // search through child nodes
            for (int pos = nodeIndex; pos < end; pos++) {
                int offset = nodeStart + ((pos - nodeIndex) * NODE_ITEM_LEN);
                skip = offset - dataPos;
                if (skip > 0) {
                    skipNBytes(data, skip);
                    dataPos += skip;
                }
                double nodeMinX = data.readDouble();
                dataPos += 8;
                if (maxX < nodeMinX) continue;
                double nodeMinY = data.readDouble();
                dataPos += 8;
                if (maxY < nodeMinY) continue;
                double nodeMaxX = data.readDouble();
                dataPos += 8;
                if (minX > nodeMaxX) continue;
                double nodeMaxY = data.readDouble();
                dataPos += 8;
                if (minY > nodeMaxY) continue;
                long indexOffset = data.readLong();
                dataPos += 8;
                if (isLeafNode) searchResult.hits.add(new SearchHit(indexOffset, pos - 1));
                else stack.add(new StackItem(indexOffset, level - 1));
            }
            stack.sort((StackItem a, StackItem b) -> (int) (b.nodeIndex - a.nodeIndex));
        }
        searchResult.pos = dataPos;
        return searchResult;
    }

    static void skipNBytes(InputStream stream, long skip) throws IOException {
        long actual = 0;
        long remaining = skip;
        while (actual < remaining) {
            remaining -= stream.skip(remaining);
        }
    }
}

package org.wololo.flatgeobuf;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.Stack;
import java.util.LinkedList;
import java.util.Collections;
import java.util.Objects;

import org.locationtech.jts.geom.Envelope;

import com.google.common.io.LittleEndianDataInputStream;

public class PackedRTree {
    private static int NODE_ITEM_LEN = 8 * 4 + 8;
    final static int HILBERT_MAX = (1 << 16) - 1;
    private int numItems;
    private int nodeSize;
    public NodeItem[] nodeItems;
    private long numNodes;
    private List<Pair<Integer, Integer>> levelBounds;

    public PackedRTree(final List<? extends Item> items, final short nodeSize) {
        this.numItems = items.size();
        init(nodeSize);
        int k = (int) (this.numNodes - (long) this.numItems);
        Iterator<? extends Item> it = items.iterator();
        for (int i = 0; i < this.numItems; ++i) {
            this.nodeItems[k++] = it.next().nodeItem;
        }
        generateNodes();
    }

    public void init(int nodeSize) {
        if (nodeSize < 2)
            throw new RuntimeException("Node size must be at least 2");
        if (numItems == 0)
            throw new RuntimeException("Number of items must be greater than 0");
        this.nodeSize = Math.min(Math.max(2, nodeSize), HILBERT_MAX);
        this.levelBounds = generateLevelBounds(numItems, this.nodeSize);
        this.numNodes = levelBounds.get(0).second;
        this.nodeItems = new NodeItem[Math.toIntExact(numNodes)];
    }

    void generateNodes() {
        long pos;
        long end = 0;
        for (short i = 0; i < levelBounds.size() - 1; i++) {
            pos = levelBounds.get(i).first;
            end = levelBounds.get(i).second;
            long newpos = levelBounds.get(i + 1).first;
            while (pos < end) {
                NodeItem node = new NodeItem(pos);
                for (short j = 0; j < this.nodeSize && pos < end; j++)
                    node.expand(nodeItems[(int) pos++]);
                nodeItems[(int) newpos++] = node;
            }
        }
    }

    public static List<? extends Item> hilbertSort(List<? extends Item> items, NodeItem extent) {
        double minX = extent.minX;
        double minY = extent.minY;
        double width = extent.width();
        double height = extent.height();
        Collections.sort(items, (a, b) -> {
            long ha = hibert(a.nodeItem, HILBERT_MAX, minX, minY, width, height);
            long hb = hibert(b.nodeItem, HILBERT_MAX, minX, minY, width, height);
            return (ha - hb) > 0 ? 1 : (ha - hb) == 0 ? 0 : -1;
        });
        return items;
    }

    public static long hibert(NodeItem nodeItem, int hilbertMax, double minX, double minY, double width, double height) {
        long x = 0;
        long y = 0;
        if (width != 0.0)
            x = (long) Math.floor(hilbertMax * ((nodeItem.minX + nodeItem.maxX) / 2 - minX) / width);
        if (height != 0.0)
            y = (long) Math.floor(hilbertMax * ((nodeItem.minY + nodeItem.maxY) / 2 - minY) / height);
        return hibert(x, y);
    }

    // Based on public domain code at https://github.com/rawrunprotected/hilbert_curves
    private static long hibert(long x, long y) {
        long a = x ^ y;
        long b = 0xFFFF ^ a;
        long c = 0xFFFF ^ (x | y);
        long d = x & (y ^ 0xFFFF);
        long A = a | (b >> 1);
        long B = (a >> 1) ^ a;
        long C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
        long D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

        a = A;
        b = B;
        c = C;
        d = D;
        A = ((a & (a >> 2)) ^ (b & (b >> 2)));
        B = ((a & (b >> 2)) ^ (b & ((a ^ b) >> 2)));
        C ^= ((a & (c >> 2)) ^ (b & (d >> 2)));
        D ^= ((b & (c >> 2)) ^ ((a ^ b) & (d >> 2)));

        a = A;
        b = B;
        c = C;
        d = D;
        A = ((a & (a >> 4)) ^ (b & (b >> 4)));
        B = ((a & (b >> 4)) ^ (b & ((a ^ b) >> 4)));
        C ^= ((a & (c >> 4)) ^ (b & (d >> 4)));
        D ^= ((b & (c >> 4)) ^ ((a ^ b) & (d >> 4)));

        a = A;
        b = B;
        c = C;
        d = D;
        C ^= ((a & (c >> 8)) ^ (b & (d >> 8)));
        D ^= ((b & (c >> 8)) ^ ((a ^ b) & (d >> 8)));

        a = C ^ (C >> 1);
        b = D ^ (D >> 1);

        long i0 = x ^ y;
        long i1 = b | (0xFFFF ^ (i0 | a));

        i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
        i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
        i0 = (i0 | (i0 << 2)) & 0x33333333;
        i0 = (i0 | (i0 << 1)) & 0x55555555;

        i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
        i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
        i1 = (i1 | (i1 << 2)) & 0x33333333;
        i1 = (i1 | (i1 << 1)) & 0x55555555;

        long value = ((i1 << 1) | i0);

        return value;
    }

    public static NodeItem calcExtent(List<? extends Item> items) {
        return items.stream().map(item -> item.nodeItem).reduce(new NodeItem(0), (nodeItem, nodeItem2) -> nodeItem.expand(nodeItem2));
    }

    public void write(OutputStream outputStream) {
        // nodeItem 40 Byte
        ByteBuffer buffer = ByteBuffer.allocate((int) (NODE_ITEM_LEN * numNodes));
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        for (NodeItem nodeItem : nodeItems) {
            buffer.putDouble(nodeItem.minX);
            buffer.putDouble(nodeItem.minY);
            buffer.putDouble(nodeItem.maxX);
            buffer.putDouble(nodeItem.maxY);
            buffer.putLong(nodeItem.offset);
        }
        buffer.flip();
        try {
            if (buffer.hasRemaining()) {
                byte[] arr = new byte[buffer.remaining()];
                buffer.get(arr);
                outputStream.write(arr);
                outputStream.flush();
            }
        } catch (IOException e) {
            throw new RuntimeException(e);
        } finally {
            buffer.clear();
            buffer = null;
        }
    }

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

    static List<Pair<Integer, Integer>> generateLevelBounds(int numItems, int nodeSize) {
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
        List<Pair<Integer, Integer>> levelBounds = new LinkedList<>();
        // bounds per level in reversed storage order (top-down)
        for (int i = 0; i < levelNumNodes.size(); i++)
            levelBounds.add(new Pair<Integer, Integer>(levelOffsets.get(i), levelOffsets.get(i) + levelNumNodes.get(i)));
        return levelBounds;
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

    public static SearchResult search(InputStream stream, int start, int numItems, int nodeSize, Envelope rect) throws IOException {
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
                if (maxX < nodeMinX)
                    continue;
                double nodeMinY = data.readDouble();
                dataPos += 8;
                if (maxY < nodeMinY)
                    continue;
                double nodeMaxX = data.readDouble();
                dataPos += 8;
                if (minX > nodeMaxX)
                    continue;
                double nodeMaxY = data.readDouble();
                dataPos += 8;
                if (minY > nodeMaxY)
                    continue;
                long indexOffset = data.readLong();
                dataPos += 8;
                if (isLeafNode)
                    searchResult.hits.add(new SearchHit(indexOffset, pos - 1));
                else
                    stack.add(new StackItem(indexOffset, level - 1));
            }
            stack.sort((StackItem a, StackItem b) -> (int) (b.nodeIndex - a.nodeIndex));
        }
        searchResult.pos = dataPos;
        return searchResult;
    }

    public static long[] readFeatureOffsets(
            LittleEndianDataInputStream data, long[] fids, HeaderMeta headerMeta)
            throws IOException {
        long treeSize = calcSize((int) headerMeta.featuresCount, headerMeta.indexNodeSize);
        List<Pair<Integer, Integer>> levelBounds =
                generateLevelBounds((int) headerMeta.featuresCount, headerMeta.indexNodeSize);
        long bottomLevelOffset = levelBounds.get(0).first * 40;

        long pos = 0;
        long[] featureOffsets = new long[fids.length];
        for (int i = 0; i < fids.length; i++) {
            if (fids[i] > headerMeta.featuresCount - 1) throw new NoSuchElementException();
            long nodeItemOffset = bottomLevelOffset + (fids[i] * 40);
            long delta = nodeItemOffset + (8 * 4) - pos;
            skipNBytes(data, delta);
            long featureOffset = data.readLong();
            pos += delta + 8;
            featureOffsets[i] = featureOffset;
        }
        long remainingIndexOffset = treeSize - pos;
        skipNBytes(data, remainingIndexOffset);

        return featureOffsets;
    }

    static void skipNBytes(InputStream stream, long skip) throws IOException {
        long actual = 0;
        long remaining = skip;
        while (actual < remaining)
            remaining -= stream.skip(remaining);
    }

    public static class Item {
        public NodeItem nodeItem;
    }

    public static class FeatureItem extends Item {
        public long size;
        public long offset;
    }

    static class Pair<T, U> {
        public T first;
        public U second;

        public Pair(T first, U second) {
            this.first = first;
            this.second = second;
        }

        @Override
        public boolean equals(Object o) {
            if (this == o)
                return true;
            if (o == null || getClass() != o.getClass())
                return false;
            Pair<?, ?> pair = (Pair<?, ?>) o;
            return Objects.equals(first, pair.first) && Objects.equals(second, pair.second);
        }

        @Override
        public int hashCode() {
            return Objects.hash(first, second);
        }
    }
}


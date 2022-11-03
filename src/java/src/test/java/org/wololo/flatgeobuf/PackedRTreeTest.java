package org.wololo.flatgeobuf;

import static org.junit.Assert.assertEquals;

import java.io.*;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;

import org.junit.Before;
import org.junit.Test;
import org.locationtech.jts.geom.Envelope;
import org.wololo.flatgeobuf.PackedRTree.SearchHit;
import org.wololo.flatgeobuf.PackedRTree.SearchResult;

public class PackedRTreeTest {

    List<PackedRTree.FeatureItem> nodeItemList = new LinkedList<>();
    List<PackedRTree.FeatureItem> sortNodeItemList = new LinkedList<>();
    NodeItem extend = new NodeItem(0);

    @Before
    public void setUp() throws Exception {
        PackedRTree.FeatureItem featureItem = new PackedRTree.FeatureItem();
        featureItem.nodeItem = new NodeItem(2.1, 2.1, 8.5, 5.5, 1000);
        PackedRTree.FeatureItem featureItem2 = new PackedRTree.FeatureItem();
        nodeItemList.add(featureItem);
        featureItem2.nodeItem = new NodeItem(10, 2.1, 12, 5.5, 500);
        nodeItemList.add(featureItem2);
        PackedRTree.FeatureItem featureItem3 = new PackedRTree.FeatureItem();
        featureItem3.nodeItem = new NodeItem(10, 3, 12, 6, 200);
        nodeItemList.add(featureItem3);
        sortNodeItemList.add(featureItem);
        sortNodeItemList.add(featureItem3);
        sortNodeItemList.add(featureItem2);
        nodeItemList.forEach(x -> extend.expand(x.nodeItem));
    }

    @Test
    public void testHilbertSort() {
        assertEquals(sortNodeItemList, PackedRTree.hilbertSort(nodeItemList, extend));
        assertEquals(sortNodeItemList, PackedRTree.hilbertSort(nodeItemList, extend));
    }

    @Test
    public void testCalcExtent() {
        assertEquals(extend, PackedRTree.calcExtent(nodeItemList));
        assertEquals(extend, PackedRTree.calcExtent(sortNodeItemList));
    }

    @Test
    public void testGenerateLevelBounds() {
        List<PackedRTree.Pair<Integer, Integer>> list = new LinkedList<>();
        list.add(new PackedRTree.Pair<>(3, 23));
        list.add(new PackedRTree.Pair<>(1, 3));
        list.add(new PackedRTree.Pair<>(0, 1));
        assertEquals(list, PackedRTree.generateLevelBounds(20, 16));
        list = new LinkedList<>();
        list.add(new PackedRTree.Pair<>(1, 17));
        list.add(new PackedRTree.Pair<>(0, 1));
        assertEquals(list, PackedRTree.generateLevelBounds(16, 16));
    }

    @Test
    public void testWrite() throws IOException {
        File tmpFile = new File("../../test/data/tmp20221102.fgb");
        tmpFile.deleteOnExit();
        tmpFile.createNewFile();
        try (FileOutputStream outputStream = new FileOutputStream(tmpFile);) {
            PackedRTree packedRTree = new PackedRTree(sortNodeItemList, extend, (short) 16);
            packedRTree.write(outputStream);
        }
        try (FileInputStream fileInputStream = new FileInputStream(tmpFile)) {
            SearchResult searchResult = PackedRTree.search(fileInputStream, 0, 3, 16, new Envelope(10, 12, 2.1, 2.999));
            assertEquals(1, searchResult.hits.size());
            assertEquals(sortNodeItemList.get(2).nodeItem.offset, searchResult.hits.get(0).offset);
        }
        tmpFile.deleteOnExit();
    }

    @Test
    public void BasicByteBuffer() throws IOException {

        File file = new File("../../test/data/countries.fgb");
        byte[] bytes = Files.readAllBytes(file.toPath());
        ByteBuffer bb = ByteBuffer.wrap(bytes);
        bb.order(ByteOrder.LITTLE_ENDIAN);

        HeaderMeta headerMeta = HeaderMeta.read(bb);

        Envelope env = new Envelope(12, 13, 56, 57);

        ArrayList<SearchHit> result = PackedRTree.search(bb, headerMeta.offset, (int) headerMeta.featuresCount, headerMeta.indexNodeSize, env);

        assertEquals(3, result.size());
    }

    @Test
    public void BasicStream() throws IOException {

        File file = new File("../../test/data/countries.fgb");
        byte[] bytes = Files.readAllBytes(file.toPath());
        ByteBuffer bb = ByteBuffer.wrap(bytes);
        bb.order(ByteOrder.LITTLE_ENDIAN);

        HeaderMeta headerMeta = HeaderMeta.read(bb);

        Envelope env = new Envelope(12, 13, 56, 57);

        long size = PackedRTree.calcSize((int) headerMeta.featuresCount, headerMeta.indexNodeSize);

        byte[] treeBytes = new byte[(int) size];
        bb.get(treeBytes, 0, (int) size);
        InputStream stream = new ByteArrayInputStream(treeBytes);

        SearchResult result = PackedRTree.search(stream, headerMeta.offset, (int) headerMeta.featuresCount, headerMeta.indexNodeSize, env);

        assertEquals(3, result.hits.size());
    }
}

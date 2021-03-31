package org.wololo.flatgeobuf.test;

import static org.junit.Assert.assertEquals;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.ArrayList;

import org.junit.Test;
import org.locationtech.jts.geom.Envelope;
import org.wololo.flatgeobuf.HeaderMeta;
import org.wololo.flatgeobuf.PackedRTree;
import org.wololo.flatgeobuf.PackedRTree.SearchHit;

public class PackedRTreeTest {
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

        ArrayList<SearchHit> result = PackedRTree.search(stream, headerMeta.offset, (int) headerMeta.featuresCount, headerMeta.indexNodeSize, env);

        assertEquals(3, result.size());
    }
}

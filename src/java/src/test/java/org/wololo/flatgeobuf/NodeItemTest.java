package org.wololo.flatgeobuf;

import org.junit.Before;
import org.junit.Test;
import org.locationtech.jts.geom.Envelope;

import static org.junit.Assert.*;

public class NodeItemTest {
    NodeItem nodeItem;
    NodeItem nodeItem2;
    NodeItem nodeItem3;

    @Before
    public void setUp() throws Exception {
        nodeItem = new NodeItem(500);
        nodeItem2 = new NodeItem(1.1, 1.1, 5.5, 5.5);
        nodeItem3 = new NodeItem(2.1, 2.1, 8.5, 5.5, 1000);
    }

    @Test
    public void testWidth() {
        assertEquals(4.4, nodeItem2.width(), 1E-10);
        assertEquals(3.4, nodeItem3.height(), 1E-10);
    }

    @Test
    public void testHeight() {
        assertEquals(4.4, nodeItem2.height(), 1E-10);
        assertEquals(3.4, nodeItem3.height(), 1E-10);
    }

    @Test
    public void testSum() {
        nodeItem = NodeItem.sum(nodeItem, nodeItem2);
        nodeItem.offset = nodeItem2.offset;
        assertEquals(nodeItem2, nodeItem);
        assertEquals(nodeItem2.hashCode(), nodeItem.hashCode());
        nodeItem = NodeItem.sum(nodeItem, nodeItem3);
        assertEquals(new NodeItem(1.1, 1.1, 8.5, 5.5, nodeItem.offset), nodeItem);
        assertEquals(new NodeItem(1.1, 1.1, 8.5, 5.5, nodeItem.offset).hashCode(), nodeItem.hashCode());
    }

    @Test
    public void testExpand() {
        nodeItem.expand(nodeItem2);
        nodeItem.offset = nodeItem2.offset;
        assertEquals(nodeItem2, nodeItem);
        assertEquals(nodeItem2.hashCode(), nodeItem.hashCode());
        nodeItem.expand(nodeItem3);
        assertEquals(new NodeItem(1.1, 1.1, 8.5, 5.5, nodeItem.offset), nodeItem);
        assertEquals(new NodeItem(1.1, 1.1, 8.5, 5.5, nodeItem.offset).hashCode(), nodeItem.hashCode());
    }

    @Test
    public void testIntersects() {
        assertEquals(true, nodeItem2.intersects(nodeItem3));
        assertEquals(true, nodeItem3.intersects(nodeItem2));

        assertEquals(true, nodeItem2.intersects(nodeItem2));

        assertEquals(false, nodeItem.intersects(nodeItem2));
        assertEquals(false, nodeItem2.intersects(nodeItem));

        NodeItem nodeItem4 = new NodeItem(5.51, 5.51, 8.8, 99);
        assertEquals(false, nodeItem2.intersects(nodeItem4));
        assertEquals(false, nodeItem4.intersects(nodeItem2));

        NodeItem nodeItem5 = new NodeItem(2, 5.51, 4, 99);
        assertEquals(false, nodeItem2.intersects(nodeItem5));
        assertEquals(false, nodeItem5.intersects(nodeItem2));

    }

    @Test
    public void testToEnvelope() {
        Envelope e = nodeItem2.toEnvelope();
        Envelope e2 = new Envelope(1.1, 5.5, 1.1, 5.5);
        assertEquals(e2, e);
        e = nodeItem3.toEnvelope();
        Envelope e3 = new Envelope(2.1, 8.5, 2.1, 5.5);
        assertEquals(e3, e);
    }

    @Test
    public void testTestEquals() {
        assertTrue(nodeItem2.equals(nodeItem2));
        assertFalse(nodeItem2.equals(0));
    }
}
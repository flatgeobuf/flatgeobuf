package org.wololo.flatgeobuf;

import org.locationtech.jts.geom.Envelope;

import java.util.Objects;

public class NodeItem {
    public double minX;
    public double minY;
    public double maxX;
    public double maxY;
    public long offset;

    public NodeItem(double minX, double minY, double maxX, double maxY, long offset) {
        this.minX = minX;
        this.minY = minY;
        this.maxX = maxX;
        this.maxY = maxY;
        this.offset = offset;
    }

    public NodeItem(double minX, double minY, double maxX, double maxY) {
        this(minX, minY, maxX, maxY, 0);
    }

    public NodeItem(long offset) {
        this(Double.POSITIVE_INFINITY, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NEGATIVE_INFINITY, offset);
    }

    public double width() {
        return maxX - minX;
    }

    public double height() {
        return maxY - minY;
    }

    public static NodeItem sum(NodeItem a, final NodeItem b) {
        a.expand(b);
        return a;
    }

    public NodeItem expand(final NodeItem nodeItem) {
        if (nodeItem.minX < minX) {
            minX = nodeItem.minX;
        }
        if (nodeItem.minY < minY) {
            minY = nodeItem.minY;
        }
        if (nodeItem.maxX > maxX) {
            maxX = nodeItem.maxX;
        }
        if (nodeItem.maxY > maxY) {
            maxY = nodeItem.maxY;
        }
        return this;
    }

    public boolean intersects(NodeItem nodeItem) {
        if (nodeItem.minX > maxX) {
            return false;
        }
        if (nodeItem.minY > maxY) {
            return false;
        }
        if (nodeItem.maxX < minX) {
            return false;
        }
        if (nodeItem.maxY < minY) {
            return false;
        }
        return true;
    }

    public Envelope toEnvelope() {
        return new Envelope(minX, maxX, minY, maxY);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o)
            return true;
        if (o == null || getClass() != o.getClass())
            return false;
        NodeItem nodeItem = (NodeItem) o;
        return Double.compare(nodeItem.minX, minX) == 0 && Double.compare(nodeItem.minY, minY) == 0 && Double.compare(nodeItem.maxX, maxX) == 0
                && Double.compare(nodeItem.maxY, maxY) == 0 && offset == nodeItem.offset;
    }

    @Override
    public int hashCode() {
        return Objects.hash(minX, minY, maxX, maxY, offset);
    }
}

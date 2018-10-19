#ifndef FLATGEOBUF_PACKEDHILBERTRTREE_H_
#define FLATGEOBUF_PACKEDHILBERTRTREE_H_

#include "flatbuffers/flatbuffers.h"

namespace FlatGeobuf {

struct Rect {
    double minX;
    double minY;
    double maxX;
    double maxY;

    double width() { return maxX - minX; };
    double height() { return maxY - minY; };

    static Rect createInvertedInfiniteRect() {
        return {
            std::numeric_limits<double>::infinity(),
            std::numeric_limits<double>::infinity(),
            -1 * std::numeric_limits<double>::infinity(),
            -1 * std::numeric_limits<double>::infinity()
        };
    }

    void expand(Rect r) {
        if (r.minX < minX) minX = r.minX;
        if (r.minY < minY) minY = r.minY;
        if (r.maxX > maxX) maxX = r.maxX;
        if (r.maxY > maxY) maxY = r.maxY;
    }

    bool intersects(Rect r)
    {
        if (maxX < r.minX) return false;
        if (maxY < r.minY) return false;
        if (minX > r.maxX) return false;
        if (minY > r.maxY) return false;
        return true;
    }
};

class PackedHilbertRTree {

    Rect _extent;
    std::vector<Rect> _rects;
    std::vector<u_int64_t> _indices;
    u_int64_t _pos;

    u_int64_t _numItems;
    ulong _numNodes;
    u_int16_t _nodeSize;

    std::vector<u_int64_t> _levelBounds;

public:
    PackedHilbertRTree(u_int64_t numItems, u_int16_t nodeSize = 16);
    void add(Rect r);
    void add(double minX, double minY, double maxX, double maxY);
    void finish();
    std::vector<u_int64_t> search(double minX, double minY, double maxX, double maxY);
    static void sort(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t left, u_int64_t right);
    static void swap(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t i, u_int64_t j);
    static u_int64_t hilbert(u_int64_t x, u_int64_t y);
    u_int64_t size() { return _numNodes * 4 * 8 + _numNodes * 8; };

    static PackedHilbertRTree fromData(u_int64_t numItems, char* data);
    static char* toData(PackedHilbertRTree packedHilbertRTree);
};

}

#endif
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
    static Rect createInvertedInfiniteRect();
    void expand(Rect r);
    bool intersects(Rect r);
    std::vector<double> toVector();
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
    static void sort(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t left, u_int64_t right);
    static void swap(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t i, u_int64_t j);
    static u_int64_t hilbert(u_int64_t x, u_int64_t y);
public:
    PackedHilbertRTree(const u_int64_t numItems, const u_int16_t nodeSize = 16, const void* data = nullptr);
    void add(Rect r);
    void add(double minX, double minY, double maxX, double maxY);
    void finish();
    std::vector<u_int64_t> search(double minX, double minY, double maxX, double maxY);
    u_int64_t size() { return _numNodes * 4 * 8 + _numNodes * 8; };
    u_int8_t* toData();
    Rect getExtent() { return _extent; };
    std::vector<u_int64_t> getIndices() { return _indices; };
};

}

#endif
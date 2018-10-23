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
    std::vector<uint64_t> _indices;
    uint64_t _pos;
    uint64_t _numItems;
    ulong _numNodes;
    uint16_t _nodeSize;
    std::vector<uint64_t> _levelBounds;
    static void sort(std::vector<uint64_t> &values, std::vector<Rect> &boxes, std::vector<uint64_t> &indices, uint64_t left, uint64_t right);
    static void swap(std::vector<uint64_t> &values, std::vector<Rect> &boxes, std::vector<uint64_t> &indices, uint64_t i, uint64_t j);
    static uint64_t hilbert(uint64_t x, uint64_t y);
public:
    PackedHilbertRTree();
    PackedHilbertRTree(const uint64_t numItems, const uint16_t nodeSize = 16, const void* data = nullptr);
    void init(const uint64_t numItems, const uint16_t nodeSize = 16, const void* data = nullptr);
    void add(Rect r);
    void add(double minX, double minY, double maxX, double maxY);
    void finish();
    std::vector<uint64_t> search(double minX, double minY, double maxX, double maxY);
    uint64_t size() { return _numNodes * 4 * 8 + _numNodes * 8; };
    uint8_t* toData();
    Rect getExtent() { return _extent; };
    std::vector<uint64_t> getIndices() { return _indices; };
};

}

#endif

#ifndef FLATGEOBUF_PACKEDRTREE_H_
#define FLATGEOBUF_PACKEDRTREE_H_

#include <cmath>
#include <numeric>

#include "flatbuffers/flatbuffers.h"

namespace FlatGeobuf {

struct Rect {
    double minX;
    double minY;
    double maxX;
    double maxY;
    double width() { return maxX - minX; }
    double height() { return maxY - minY; }
    static Rect sum(Rect a, Rect b) {
        a.expand(b);
        return a;
    }
    static Rect createInvertedInfiniteRect();
    void expand(Rect r);
    bool intersects(Rect r) const;
    std::vector<double> toVector();
};

struct Item {
    Rect rect;
};

std::ostream& operator << (std::ostream& os, Rect const& value);

uint32_t hilbert(uint32_t x, uint32_t y);
uint32_t hilbert(Rect r, uint32_t hilbertMax, Rect extent);
void hilbertSort(std::vector<Item *> &items);
void hilbertSort(std::vector<Rect> &items);
Rect calcExtent(std::vector<Item *> &rectitems);
Rect calcExtent(std::vector<Rect> &rects);

/**
 * Packed R-Tree
 * Based on https://github.com/mourner/flatbush
 */
class PackedRTree {
    Rect _extent;
    std::vector<Rect> _rects;
    std::vector<uint32_t> _indices;
    uint64_t _numItems;
    uint64_t _numNodes;
    uint64_t _numNonLeafNodes;
    uint16_t _nodeSize;
    std::vector<uint64_t> _levelBounds;
    void init(const uint16_t nodeSize);
    static std::vector<uint64_t> generateLevelBounds(const uint64_t numItems, const uint16_t nodeSize);
    void generateNodes();
    void fromData(const void *data);
public:
    PackedRTree(std::vector<Item *> &items, Rect extent, const uint16_t nodeSize = 16);
    PackedRTree(std::vector<Rect> &rects, Rect extent, const uint16_t nodeSize = 16);
    PackedRTree(const void *data, const uint64_t numItems, const uint16_t nodeSize = 16);
    std::vector<uint64_t> search(double minX, double minY, double maxX, double maxY) const;
    static std::vector<uint64_t> streamSearch(
        const uint64_t numItems, const uint16_t nodeSize, Rect r,
        const std::function<void(uint8_t *, uint32_t, uint32_t)> &readNode);
    uint64_t size() const;
    static uint64_t size(const uint64_t numItems, const uint16_t nodeSize = 16);
    uint8_t *toData() const;
    Rect getExtent() const;
};

}

#endif

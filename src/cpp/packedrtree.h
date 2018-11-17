#ifndef FLATGEOBUF_PACKEDRTREE_H_
#define FLATGEOBUF_PACKEDRTREE_H_

#include <cmath>
#include <stack>

#include "flatbuffers/flatbuffers.h"

namespace FlatGeobuf {

struct Rect {
    double minX;
    double minY;
    double maxX;
    double maxY;
    double width() { return maxX - minX; }
    double height() { return maxY - minY; }
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
    static Rect sum(Rect a, Rect b) {
        a.expand(b);
        return a;
    }
    bool intersects(Rect r) const {
        if (maxX < r.minX) return false;
        if (maxY < r.minY) return false;
        if (minX > r.maxX) return false;
        if (minY > r.maxY) return false;
        return true;
    }
    std::vector<double> toVector() {
        return std::vector<double> { minX, minY, maxX, maxY };
    }
};

std::ostream& operator << ( std::ostream& os, Rect const& value ) {
    os << std::to_string(value.minX) << " "
       << std::to_string(value.minY) << " "
       << std::to_string(value.maxX) << " "
       << std::to_string(value.maxY);
    return os;
}

template <class T>
T hilbert(T x, T y) {
    T a = x ^ y;
    T b = 0xFFFF ^ a;
    T c = 0xFFFF ^ (x | y);
    T d = x & (y ^ 0xFFFF);

    T A = a | (b >> 1);
    T B = (a >> 1) ^ a;
    T C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    T D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

    a = A; b = B; c = C; d = D;
    A = ((a & (a >> 2)) ^ (b & (b >> 2)));
    B = ((a & (b >> 2)) ^ (b & ((a ^ b) >> 2)));
    C ^= ((a & (c >> 2)) ^ (b & (d >> 2)));
    D ^= ((b & (c >> 2)) ^ ((a ^ b) & (d >> 2)));

    a = A; b = B; c = C; d = D;
    A = ((a & (a >> 4)) ^ (b & (b >> 4)));
    B = ((a & (b >> 4)) ^ (b & ((a ^ b) >> 4)));
    C ^= ((a & (c >> 4)) ^ (b & (d >> 4)));
    D ^= ((b & (c >> 4)) ^ ((a ^ b) & (d >> 4)));

    a = A; b = B; c = C; d = D;
    C ^= ((a & (c >> 8)) ^ (b & (d >> 8)));
    D ^= ((b & (c >> 8)) ^ ((a ^ b) & (d >> 8)));

    a = C ^ (C >> 1);
    b = D ^ (D >> 1);

    T i0 = x ^ y;
    T i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    auto value = ((i1 << 1) | i0);

    return value;
}

template <class T>
T hilbert(Rect r, T hilbertMax, Rect extent)
{
    T x = floor(hilbertMax * ((r.minX + r.maxX) / 2 - extent.minX) / extent.width());
    T y = floor(hilbertMax * ((r.minY + r.maxY) / 2 - extent.minY) / extent.height());
    T v = hilbert(x, y);
    return v;
}

template <class T, class T2>
std::function<bool (T2, T2)> createHilbertCompare(Rect extent, std::function<Rect (T2)> getRect)
{
    T hilbertMax = (1 << 16) - 1;
    auto compare = [hilbertMax, &extent, &getRect] (T2 a, T2 b) {
        Rect ra = getRect(a);
        Rect rb = getRect(b);
        T ha = hilbert(ra, hilbertMax, extent);
        T hb = hilbert(rb, hilbertMax, extent);
        return ha > hb;
    };
    return compare;
}

template <class T, class T2>
void hilbertSort(std::vector<T2> &items, std::function<Rect (T2)> getRect)
{
    Rect extent = std::accumulate(items.begin(), items.end(), Rect::createInvertedInfiniteRect(), Rect::sum);
    std::sort(items.begin(), items.end(), createHilbertCompare<T>(extent, getRect));
}

template <class T>
void hilbertSort(std::vector<Rect> &items)
{
    hilbertSort<T, Rect>(items, [] (Rect r) { return r; });
}


Rect calcExtent(std::vector<Rect> &rects)
{
    Rect extent = std::accumulate(rects.begin(), rects.end(), Rect::createInvertedInfiniteRect(), Rect::sum);
    return extent;
}

/**
 * Packed Hilbert R-Tree
 * Based on https://github.com/mourner/flatbush
 */
template <class T>
class PackedRTree {
    Rect _extent;
    std::vector<Rect> _rects;
    std::vector<T> _indices;
    T _numItems;
    T _numNodes;
    T _numNonLeafNodes;
    uint16_t _nodeSize;
    std::vector<T> _levelBounds;
    void init(const uint16_t nodeSize) {
        if (_numItems == 0)
            throw std::invalid_argument("Cannot create empty tree");

        _nodeSize = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));

        T n = _numItems;
        T numNodes = n;
        _levelBounds.push_back(n);
        do {
            n = (n + _nodeSize - 1) / _nodeSize;
            numNodes += n;
            _levelBounds.push_back(numNodes);
        } while (n != 1);

        _numNodes = numNodes;
        _numNonLeafNodes = _numNodes - _numItems;

        _rects.reserve(_numNodes);
        _indices.reserve(_numNonLeafNodes);
    }
    void generateNodes() {
        for (T i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
            T end = _levelBounds[i];
            while (pos < end) {
                Rect nodeRect = Rect::createInvertedInfiniteRect();
                T nodeIndex = pos;
                for (T j = 0; j < _nodeSize && pos < end; j++)
                    nodeRect.expand(_rects[pos++]);
                _rects.push_back(nodeRect);
                _indices.push_back(nodeIndex);
            }
        }
    }
    void fromData(const void *data) {
        auto buf = reinterpret_cast<const uint8_t *>(data);
        const Rect *pr = reinterpret_cast<const Rect*>(buf);
        for (T i = 0; i < _numNodes; i++) {
            Rect r = *pr++;
            _rects.push_back(r);
            _extent.expand(r);
        }
        uint64_t rectsSize = _numNodes * sizeof(Rect);
        const T *pi = reinterpret_cast<const T*>(buf + rectsSize);
        for (T i = 0; i < _numNonLeafNodes; i++)
            _indices[i] = *pi++;
    }
public:
    PackedRTree(std::vector<Rect> &rects, Rect extent, const uint16_t nodeSize = 16) :
        _extent(extent),
        _rects(rects),
        _numItems(_rects.size())
    {
        init(nodeSize);
        generateNodes();
    }
    PackedRTree(const void *data, const T numItems, const uint16_t nodeSize = 16) :
        _extent(Rect::createInvertedInfiniteRect()),
        _numItems(numItems)
    {
        init(nodeSize);
        fromData(data);
    }
    std::vector<T> search(double minX, double minY, double maxX, double maxY) const {
        Rect r { minX, minY, maxX, maxY };
        std::vector<T> queue;
        std::vector<T> results;
        queue.push_back(_rects.size() - 1);
	    queue.push_back(_levelBounds.size() - 1);
        while(queue.size() != 0) {
            T nodeIndex = queue[queue.size() - 2];
            T level = queue[queue.size() - 1];
            queue.pop_back();
            queue.pop_back();
            // find the end index of the node
            T end = std::min(static_cast<T>(nodeIndex + _nodeSize), _levelBounds[level]);
            // search through child nodes
            for (T pos = nodeIndex; pos < end; pos++) {
                T index = pos < _numItems ? pos : _indices[pos - _numItems];
                if (!r.intersects(_rects[pos]))
                    continue;
                if (nodeIndex < _numItems) {
                    results.push_back(index); // leaf item
                } else {
                    queue.push_back(index); // node; add it to the search queue
                    queue.push_back(level - 1);
                }
            }
        }
        return results;
    }
    uint64_t size() const { return _numNodes * sizeof(Rect) + _numNonLeafNodes * sizeof(T); }
    uint8_t *toData() const {
        T rectsSize = _numNodes * sizeof(Rect);
        T indicesSize = _numNonLeafNodes * sizeof(T);
        uint8_t *data = new uint8_t[rectsSize + indicesSize];
        Rect *pr = reinterpret_cast<Rect *>(data);
        for (T i = 0; i < _numNodes; i++)
            *pr++ = _rects[i];
        T *pi = reinterpret_cast<T *>(data + rectsSize);
        for (T i = 0; i < _numNonLeafNodes; i++)
            *pi++ = _indices[i];
        return data;
    }
    Rect getExtent() const { return _extent; }
};

}

#endif

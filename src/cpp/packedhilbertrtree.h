#ifndef FLATGEOBUF_PACKEDHILBERTRTREE_H_
#define FLATGEOBUF_PACKEDHILBERTRTREE_H_

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

template <class T>
class PackedHilbertRTree {
    Rect _extent;
    std::vector<Rect> _rects;
    std::vector<T> _indices;
    T _pos;
    T _numItems;
    T _numNodes;
    uint16_t _nodeSize;
    std::vector<T> _levelBounds;
    static void sort(std::vector<T> &values, std::vector<Rect> &boxes, std::vector<T> &indices, T left, T right) {
        if (left >= right) return;

        T pivot = values[(left + right) >> 1];
        T i = left - 1;
        T j = right + 1;

        while (true) {
            do i++; while (values[i] < pivot);
            do j--; while (values[j] > pivot);
            if (i >= j) break;
            swap(values, boxes, indices, i, j);
        }

        sort(values, boxes, indices, left, j);
        sort(values, boxes, indices, j + 1, right);
    }
    static void swap(std::vector<T> &values, std::vector<Rect> &boxes, std::vector<T> &indices, T i, T j) {
        T temp = values[i];
        values[i] = values[j];
        values[j] = temp;

        auto r = boxes[i];
        boxes[i] = boxes[j];
        boxes[j] = r;

        T e = indices[i];
        indices[i] = indices[j];
        indices[j] = e;
    }
    static T hilbert(T x, T y) {
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

        return ((i1 << 1) | i0);
    }
public:
    PackedHilbertRTree(const T numItems, const uint16_t nodeSize = 16, const void *data = nullptr) {
        if (numItems == 0)
            throw std::invalid_argument("Cannot create empty tree");

        _pos = 0;
        _extent = Rect::createInvertedInfiniteRect();

        _numItems = numItems;
        _nodeSize = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));

        T n = numItems;
        T numNodes = n;
        _levelBounds = std::vector<T> { n };
        do {
            n = ceil(static_cast<double>(n) / _nodeSize);
            numNodes += n;
            _levelBounds.push_back(numNodes);
        } while (n != 1);

        _numNodes = numNodes;

        _rects.reserve(_numNodes);
        _indices.reserve(_numNodes);

        if (data != nullptr) {
            auto buf = reinterpret_cast<const uint8_t*>(data);
            uint64_t rectsSize = _numNodes * sizeof(Rect);
            const Rect* pr = reinterpret_cast<const Rect*>(buf);
            const T* pi = reinterpret_cast<const T*>(buf + rectsSize);
            for (T i = 0; i < _numNodes; i++) {
                add(*pr++);
                _indices[i] = *pi++;
            }
        }
    }
    static uint64_t calcNumNodes(const uint64_t numItems, const uint16_t nodeSize = 16) {
        auto n = numItems;
        auto numNodes = n;
        do {
            n = ceil(static_cast<double>(n) / nodeSize);
            numNodes += n;
        } while (n != 1);
        return numNodes;
    }
    void replaceRootIndices(const std::vector<T> rootIndices) {
        std::copy(rootIndices.begin(), rootIndices.end(), _indices.begin());
    }
    void add(Rect r) {
        _indices.push_back(_pos++);
        _rects.push_back(r);
        _extent.expand(r);
    }
    void add(double minX, double minY, double maxX, double maxY) {
        add(Rect { minX, minY, maxX, maxY });
    }
    void finish() {
        if (_pos != _numItems)
            throw std::runtime_error("_pos != _numItems");

        T hilbertMax = (1 << 16) - 1;

        // map item centers into Hilbert coordinate space and calculate Hilbert values
        std::vector<T> hilbertValues(_numItems);
        for (T i = 0; i < _numItems; i++) {
            auto r = _rects[i];
            T x = floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.width());
            T y = floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.height());
            hilbertValues.push_back(hilbert(x, y));
        }

        // sort items by their Hilbert value (for packing later)
        sort(hilbertValues, _rects, _indices, 0, _numItems - 1);

        // generate nodes at each tree level, bottom-up
        for (T i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
            T end = _levelBounds[i];
            while (pos < end) {
                auto nodeRect = Rect::createInvertedInfiniteRect();
                T nodeIndex = pos;
                for (T j = 0; j < _nodeSize && pos < end; j++)
                    nodeRect.expand(_rects[pos++]);
                _rects.push_back(nodeRect);
                _indices.push_back(nodeIndex);
                _pos++;
            }
        }
    }
    std::vector<T> search(double minX, double minY, double maxX, double maxY) const {
        if (_pos != _rects.size())
            throw std::runtime_error("Data not yet indexed - call finish().");

        Rect r { minX, minY, maxX, maxY };

        T nodeIndex = _rects.size() - 1;
        T level = _levelBounds.size() - 1;
        std::stack<T> stack;
        std::vector<T> results;

        //bool cont = true;
        while(true) {
            // find the end index of the node
            T end = std::min(static_cast<T>(nodeIndex + _nodeSize), _levelBounds[level]);

            // search through child nodes
            for (T pos = nodeIndex; pos < end; pos++) {
                T index = _indices[pos];

                // check if node bbox intersects with query bbox
                if (!r.intersects(_rects[pos]))
                    continue;

                if (nodeIndex < _numItems) {
                    results.push_back(index); // leaf item
                } else {
                    stack.push(index); // node; add it to the search queue
                    stack.push(level - 1);
                }
            }

            if (stack.size() == 0)
                break;
            level = stack.top();
            stack.pop();
            nodeIndex = stack.top();
            stack.pop();
            //if (stack.size() == 0)
            //    cont = false;
        }

        return results;
    }
    uint64_t numNodes() const { return _numNodes; }
    uint64_t size() const { return _numNodes * sizeof(Rect) + _numNodes * sizeof(T); }
    uint8_t* toData() const {
        T rectsSize = _numNodes * sizeof(Rect);
        T indicesSize = _numNodes * sizeof(T);
        uint8_t *data = new uint8_t[rectsSize + indicesSize];
        Rect *pr = reinterpret_cast<Rect *>(data);
        T *pi = reinterpret_cast<T *>(data + rectsSize);
        for (T i = 0; i < _numNodes; i++) {
            *pr++ = _rects[i];
            *pi++ = _indices[i];
        }
        return data;
    }
    Rect getExtent() const { return _extent; }
    Rect getRect(T i) const { return _rects[i]; }
    T getIndex(T i) const { return _indices[i]; }
};

}

#endif

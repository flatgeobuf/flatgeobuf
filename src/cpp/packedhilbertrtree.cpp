#include <cmath>
#include <stack>

#include "packedhilbertrtree.h"

using namespace std;
using namespace FlatGeobuf;

Rect Rect::createInvertedInfiniteRect() {
    return {
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity()
    };
}

void Rect::expand(Rect r) {
    if (r.minX < minX) minX = r.minX;
    if (r.minY < minY) minY = r.minY;
    if (r.maxX > maxX) maxX = r.maxX;
    if (r.maxY > maxY) maxY = r.maxY;
}

bool Rect::intersects(Rect r)
{
    if (maxX < r.minX) return false;
    if (maxY < r.minY) return false;
    if (minX > r.maxX) return false;
    if (minY > r.maxY) return false;
    return true;
}


std::vector<double> Rect::toVector()
{
    return std::vector<double> { minX, minY, maxX, maxY };
}

PackedHilbertRTree::PackedHilbertRTree()
{

}

PackedHilbertRTree::PackedHilbertRTree(const uint64_t numItems, const uint16_t nodeSize, const void* data)
{
    init(numItems, nodeSize, data);
}

void PackedHilbertRTree::init(const uint64_t numItems, const uint16_t nodeSize, const void* data)
{
    if (numItems == 0)
        throw std::invalid_argument("Cannot create empty tree");

    _pos = 0;
    _extent = Rect::createInvertedInfiniteRect();

    _numItems = numItems;
    _nodeSize = min(max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));

    uint64_t n = numItems;
    uint64_t numNodes = n;
    _levelBounds = std::vector<uint64_t> { n };
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
        auto rectSize = _numNodes * 8 * 4;
        // auto indicesSize = 4 + _numNodes * 8;
        const Rect* pr = reinterpret_cast<const Rect*>(buf);
        for (uint64_t i = 0; i < _numNodes; i++)
            add(*pr++);
        const uint64_t* pi = reinterpret_cast<const uint64_t*>(buf+rectSize);
        for (uint64_t i = 0; i < _numNodes; i++)
            _indices[i] = *pi++;
    }
}

void PackedHilbertRTree::add(double minX, double minY, double maxX, double maxY)
{
    add(Rect { minX, minY, maxX, maxY });
}

void PackedHilbertRTree::add(Rect r)
{
    _indices.push_back(_pos);
    _rects.push_back(r);
    _extent.expand(r);
    _pos++;
}

void PackedHilbertRTree::finish()
{
    uint64_t hilbertMax = (1 << 16) - 1;

    // map item centers into Hilbert coordinate space and calculate Hilbert values
    std::vector<uint64_t> hilbertValues(_numItems);
    for (uint64_t i = 0; i < _numItems; i++) {
        Rect r = _rects[i];
        uint64_t x = floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.width());
        uint64_t y = floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.height());
        hilbertValues.push_back(hilbert(x, y));
    }

    // sort items by their Hilbert value (for packing later)
    sort(hilbertValues, _rects, _indices, 0, _numItems - 1);

    // generate nodes at each tree level, bottom-up
    for (uint16_t i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
        uint64_t end = _levelBounds[i];
        while (pos < end) {
            Rect nodeRect = Rect::createInvertedInfiniteRect();
            uint16_t nodeIndex = pos;
            for (uint64_t j = 0; j < _nodeSize && pos < end; j++)
                nodeRect.expand(_rects[pos++]);
            _rects.push_back(nodeRect);
            _indices.push_back(nodeIndex);
            _pos++;
        }
    }
}

std::vector<uint64_t> PackedHilbertRTree::search(double minX, double minY, double maxX, double maxY)
{
    Rect r { minX, minY, maxX, maxY };

    uint64_t nodeIndex = _rects.size() - 1;
    uint16_t level = _levelBounds.size() - 1;
    std::stack<uint64_t> stack;
    std::vector<uint64_t> results;

    while(true) {
        // find the end index of the node
        uint64_t end = min(nodeIndex + _nodeSize, _levelBounds[level]);

        // search through child nodes
        for (uint64_t pos = nodeIndex; pos < end; pos++) {
            uint64_t index = _indices[pos];

            // check if node bbox intersects with query bbox
            if (!r.intersects(_rects[pos])) continue;

            if (nodeIndex < _numItems) {
                results.push_back(index); // leaf item
            }
            else {
                stack.push(index); // node; add it to the search queue
                stack.push(level - 1);
            }
        }

        if (stack.size() == 0) break;
        level = stack.top();
        stack.pop();
        nodeIndex = stack.top();
        stack.pop();
    }

    return results;
}

// custom quicksort that sorts bbox data alongside the hilbert values
void PackedHilbertRTree::sort(std::vector<uint64_t>& values, std::vector<Rect>& boxes, std::vector<uint64_t>& indices, uint64_t left, uint64_t right)
{
    if (left >= right) return;

    uint64_t pivot = values[(left + right) >> 1];
    uint64_t i = left - 1;
    uint64_t j = right + 1;

    while (true) {
        do i++; while (values[i] < pivot);
        do j--; while (values[j] > pivot);
        if (i >= j) break;
        swap(values, boxes, indices, i, j);
    }

    sort(values, boxes, indices, left, j);
    sort(values, boxes, indices, j + 1, right);
}

// swap two values and two corresponding boxes
void PackedHilbertRTree::swap(std::vector<uint64_t>& values, std::vector<Rect>& boxes, std::vector<uint64_t> & indices, uint64_t i, uint64_t j)
{
    uint64_t temp = values[i];
    values[i] = values[j];
    values[j] = temp;

    Rect r = boxes[i];
    boxes[i] = boxes[j];
    boxes[j] = r;

    uint64_t e = indices[i];
    indices[i] = indices[j];
    indices[j] = e;
}

// Fast Hilbert curve algorithm by http://threadlocalmutex.com/
// Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
uint64_t PackedHilbertRTree::hilbert(uint64_t x, uint64_t y)
{
    uint64_t a = x ^ y;
    uint64_t b = 0xFFFF ^ a;
    uint64_t c = 0xFFFF ^ (x | y);
    uint64_t d = x & (y ^ 0xFFFF);

    uint64_t A = a | (b >> 1);
    uint64_t B = (a >> 1) ^ a;
    uint64_t C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    uint64_t D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

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

    uint64_t i0 = x ^ y;
    uint64_t i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    return ((i1 << 1) | i0) >> 0;
}

uint8_t* PackedHilbertRTree::toData()
{
    auto rectSize = _numNodes * 8 * 4;
    auto indicesSize = 4 + _numNodes * 8;
    auto data = new uint8_t[rectSize + indicesSize];
    Rect* pr = reinterpret_cast<Rect*>(data);
    for (uint64_t i = 0; i < _numNodes; i++)
        *pr++ = _rects[i];
    uint64_t* pi = (uint64_t*) (data+rectSize);
    for (uint64_t i = 0; i < _numNodes; i++)
        *pi++ = _indices[i];
    return data;
}

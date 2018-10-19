#include <cmath>
#include <stack>

#include "packedhilbertrtree.h"

using namespace std;
using namespace FlatGeobuf;

PackedHilbertRTree::PackedHilbertRTree(u_int64_t numItems, u_int16_t nodeSize)
{
    if (numItems == 0)
        throw std::invalid_argument("Cannot create empty tree");

    _pos = 0;
    _extent = Rect::createInvertedInfiniteRect();

    _numItems = numItems;
    _nodeSize = min(max(nodeSize, static_cast<u_int16_t>(2)), static_cast<u_int16_t>(65535));

    u_int64_t n = numItems;
    u_int64_t numNodes = n;
    _levelBounds = std::vector<u_int64_t> { n };
    do {
        n = ceil(static_cast<double>(n) / _nodeSize);
        numNodes += n;
        _levelBounds.push_back(numNodes);
    } while (n != 1);

    _numNodes = numNodes;

    _rects.reserve(_numNodes);
    _indices.reserve(_numNodes);
}

void PackedHilbertRTree::add(double minX, double minY, double maxX, double maxY)
{
    Rect r { minX, minY, maxX, maxY };
    add(r);
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
    u_int64_t hilbertMax = (1 << 16) - 1;

    // map item centers into Hilbert coordinate space and calculate Hilbert values
    std::vector<u_int64_t> hilbertValues(_numItems);
    for (u_int64_t i = 0; i < _numItems; i++) {
        Rect r = _rects[i];
        u_int64_t x = floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.width());
        u_int64_t y = floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.height());
        hilbertValues.push_back(hilbert(x, y));
    }

    // sort items by their Hilbert value (for packing later)
    sort(hilbertValues, _rects, _indices, 0, _numItems - 1);
    
    // generate nodes at each tree level, bottom-up
    for (u_int16_t i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
        u_int64_t end = _levelBounds[i];
        while (pos < end) {
            Rect nodeRect = Rect::createInvertedInfiniteRect();
            u_int16_t nodeIndex = pos;
            for (u_int64_t j = 0; j < _nodeSize && pos < end; j++)
                nodeRect.expand(_rects[pos++]);
            _rects.push_back(nodeRect);
            _indices.push_back(nodeIndex);
            _pos++;
        }
    }
}

std::vector<u_int64_t> PackedHilbertRTree::search(double minX, double minY, double maxX, double maxY)
{
    Rect r { minX, minY, maxX, maxY };

    u_int64_t nodeIndex = _rects.size() - 1;
    u_int16_t level = _levelBounds.size() - 1;
    std::stack<u_int64_t> stack;
    std::vector<u_int64_t> results;
    
    while(true) {
        // find the end index of the node
        u_int64_t end = min(nodeIndex + _nodeSize, _levelBounds[level]);

        // search through child nodes
        for (u_int64_t pos = nodeIndex; pos < end; pos++) {
            u_int64_t index = _indices[pos];

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
void PackedHilbertRTree::sort(std::vector<u_int64_t>& values, std::vector<Rect>& boxes, std::vector<u_int64_t>& indices, u_int64_t left, u_int64_t right)
{
    if (left >= right) return;

    u_int64_t pivot = values[(left + right) >> 1];
    u_int64_t i = left - 1;
    u_int64_t j = right + 1;

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
void PackedHilbertRTree::swap(std::vector<u_int64_t>& values, std::vector<Rect>& boxes, std::vector<u_int64_t> & indices, u_int64_t i, u_int64_t j)
{
    u_int64_t temp = values[i];
    values[i] = values[j];
    values[j] = temp;
    
    Rect r = boxes[i];
    boxes[i] = boxes[j];
    boxes[j] = r;
    
    u_int64_t e = indices[i];
    indices[i] = indices[j];
    indices[j] = e;
}

// Fast Hilbert curve algorithm by http://threadlocalmutex.com/
// Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
u_int64_t PackedHilbertRTree::hilbert(u_int64_t x, u_int64_t y)
{
    u_int64_t a = x ^ y;
    u_int64_t b = 0xFFFF ^ a;
    u_int64_t c = 0xFFFF ^ (x | y);
    u_int64_t d = x & (y ^ 0xFFFF);

    u_int64_t A = a | (b >> 1);
    u_int64_t B = (a >> 1) ^ a;
    u_int64_t C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    u_int64_t D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

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

    u_int64_t i0 = x ^ y;
    u_int64_t i1 = b | (0xFFFF ^ (i0 | a));

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

PackedHilbertRTree PackedHilbertRTree::fromData(u_int64_t numItems, char* data)
{
    PackedHilbertRTree tree(numItems);
    auto numNodes = tree._numNodes;
    auto rectSize = numNodes * 8 * 4;
    auto indicesSize = 4 + numNodes * 8;
    Rect* pr = reinterpret_cast<Rect*>(data);
    for (u_int64_t i = 0; i < numNodes; i++)
        tree.add(*pr++);
    u_int64_t* pi = reinterpret_cast<u_int64_t*>(data+rectSize);
    for (u_int64_t i = 0; i < numNodes; i++)
        tree._indices[i] = *pi++;
    return tree;
}

char* PackedHilbertRTree::toData(PackedHilbertRTree packedHilbertRTree)
{
    auto numNodes = packedHilbertRTree._numNodes;
    auto rectSize = numNodes * 8 * 4;
    auto indicesSize = 4 + numNodes * 8;
    auto data = new char[rectSize + indicesSize];
    Rect* pr = reinterpret_cast<Rect*>(data);
    for (u_int64_t i = 0; i < numNodes; i++)
        *pr++ = packedHilbertRTree._rects[i];
    u_int64_t* pi = (u_int64_t*) (data+rectSize);
    for (u_int64_t i = 0; i < numNodes; i++)
        *pi++ = packedHilbertRTree._indices[i];
    return data;
}

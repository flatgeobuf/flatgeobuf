
#include <cmath>
#include <stack>

#include "packedhilbertrtree.h"

using namespace std;
using namespace FlatGeobuf;

PackedHilbertRTree::PackedHilbertRTree(u_int64_t numItems, u_int16_t nodeSize)
{
    _extent = Rect::createInvertedInfiniteRect();

    _numItems = numItems;
    _nodeSize = min(max(nodeSize, static_cast<u_int16_t>(2)), static_cast<u_int16_t>(65535));

    auto n = numItems;
    auto numNodes = n;
    _levelBounds.reserve(n);
    do
    {
        n = (ulong) ceil(n / _nodeSize);
        numNodes += n;
        _levelBounds.push_back(numNodes);
    } while (n != 1);

    _numNodes = numNodes;

    _rects.reserve(_numNodes);
    _indices.reserve(_numNodes);
}

void PackedHilbertRTree::add(double minX, double minY, double maxX, double maxY)
{
    Rect r = { minX, minY, maxX, maxY };
    _indices[_pos] = _pos;
    _rects[_pos] = r;
    _extent.expand(r);
    _pos++;
}

void PackedHilbertRTree::finish()
{
    u_int64_t hilbertMax = (1 << 16) - 1;

    // map item centers into Hilbert coordinate space and calculate Hilbert values
    std::vector<u_int64_t> hilbertValues(_numItems);
    for (u_int64_t i = 0; i < _numItems; i++)
    {
        Rect r = _rects[i];
        u_int64_t x = floor(hilbertMax * ((r.minX + r.maxX) / 2 - _extent.minX) / _extent.width());
        u_int64_t y = floor(hilbertMax * ((r.minY + r.maxY) / 2 - _extent.minY) / _extent.height());
        hilbertValues[i] = hilbert(x, y);
    }

    // sort items by their Hilbert value (for packing later)
    sort(hilbertValues, _rects, _indices, 0, _numItems - 1);
    
    // generate nodes at each tree level, bottom-up
    for (u_int16_t i = 0, pos = 0; i < _levelBounds.size() - 1; i++)
    {
        auto end = _levelBounds[i];

        while (pos < end)
        {
            auto nodeRect = Rect::createInvertedInfiniteRect();
            auto nodeIndex = pos;
            for (auto j = 0; j < _nodeSize && pos < end; j++)
                nodeRect.expand(_rects[pos++]);
            _rects[_pos] = nodeRect;
            _indices[_pos] = nodeIndex;
            _pos++;
        }
    }
}

std::vector<u_int64_t> PackedHilbertRTree::search(double minX, double minY, double maxX, double maxY)
{
    Rect r = { minX, minY, maxX, maxY };

    u_int64_t nodeIndex = _rects.size() - 1;
    u_int16_t level = _levelBounds.size() - 1;
    std::stack<u_int64_t> stack;
    std::vector<u_int64_t> results;
    
    while(true)
    {
        // find the end index of the node
        u_int64_t end = min(nodeIndex + _nodeSize, _levelBounds[level]);

        // search through child nodes
        for (u_int64_t pos = nodeIndex; pos < end; pos++)
        {
            u_int64_t index = _indices[pos];

            // check if node bbox intersects with query bbox
            if (!r.intersects(_rects[pos])) continue;

            if (nodeIndex < _numItems)
            {
                results.push_back(index); // leaf item
            }
            else
            {
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
    }

    return results;
}

// custom quicksort that sorts bbox data alongside the hilbert values
void PackedHilbertRTree::sort(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t left, u_int64_t right)
{
    if (left >= right) return;

    auto pivot = values[(left + right) >> 1];
    auto i = left - 1;
    auto j = right + 1;

    while (true)
    {
        do i++; while (values[i] < pivot);
        do j--; while (values[j] > pivot);
        if (i >= j) break;
        swap(values, boxes, indices, i, j);
    }

    sort(values, boxes, indices, left, j);
    sort(values, boxes, indices, j + 1, right);
}

// swap two values and two corresponding boxes
void PackedHilbertRTree::swap(std::vector<u_int64_t> &values, std::vector<Rect> &boxes, std::vector<u_int64_t> &indices, u_int64_t i, u_int64_t j)
{
    auto temp = values[i];
    values[i] = values[j];
    values[j] = temp;
    
    auto r = boxes[i];
    boxes[i] = boxes[j];
    boxes[j] = r;
    
    auto e = indices[i];
    indices[i] = indices[j];
    indices[j] = e;
}

// Fast Hilbert curve algorithm by http://threadlocalmutex.com/
// Ported from C++ https://github.com/rawrunprotected/hilbert_curves (public domain)
u_int64_t PackedHilbertRTree::hilbert(u_int64_t x, u_int64_t y)
{
    auto a = x ^ y;
    auto b = 0xFFFF ^ a;
    auto c = 0xFFFF ^ (x | y);
    auto d = x & (y ^ 0xFFFF);

    auto A = a | (b >> 1);
    auto B = (a >> 1) ^ a;
    auto C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    auto D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

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

    auto i0 = x ^ y;
    auto i1 = b | (0xFFFF ^ (i0 | a));

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


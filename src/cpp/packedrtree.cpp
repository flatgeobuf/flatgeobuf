#include "packedrtree.h"

namespace FlatGeobuf
{

void Rect::expand(Rect r)
{
    if (r.minX < minX) minX = r.minX;
    if (r.minY < minY) minY = r.minY;
    if (r.maxX > maxX) maxX = r.maxX;
    if (r.maxY > maxY) maxY = r.maxY;
}

Rect Rect::createInvertedInfiniteRect()
{
    return {
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity()
    };
}

bool Rect::intersects(Rect r) const
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

std::ostream& operator << ( std::ostream& os, Rect const& value )
{
    os << std::to_string(value.minX) << " "
       << std::to_string(value.minY) << " "
       << std::to_string(value.maxX) << " "
       << std::to_string(value.maxY);
    return os;
}

uint32_t hilbert(uint32_t x, uint32_t y)
{
    uint32_t a = x ^ y;
    uint32_t b = 0xFFFF ^ a;
    uint32_t c = 0xFFFF ^ (x | y);
    uint32_t d = x & (y ^ 0xFFFF);

    uint32_t A = a | (b >> 1);
    uint32_t B = (a >> 1) ^ a;
    uint32_t C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
    uint32_t D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;

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

    uint32_t i0 = x ^ y;
    uint32_t i1 = b | (0xFFFF ^ (i0 | a));

    i0 = (i0 | (i0 << 8)) & 0x00FF00FF;
    i0 = (i0 | (i0 << 4)) & 0x0F0F0F0F;
    i0 = (i0 | (i0 << 2)) & 0x33333333;
    i0 = (i0 | (i0 << 1)) & 0x55555555;

    i1 = (i1 | (i1 << 8)) & 0x00FF00FF;
    i1 = (i1 | (i1 << 4)) & 0x0F0F0F0F;
    i1 = (i1 | (i1 << 2)) & 0x33333333;
    i1 = (i1 | (i1 << 1)) & 0x55555555;

    uint32_t value = ((i1 << 1) | i0);

    return value;
}

uint32_t hilbert(Rect r, uint32_t hilbertMax, Rect extent)
{
    uint32_t x = static_cast<uint32_t>(floor(hilbertMax * ((r.minX + r.maxX) / 2 - extent.minX) / extent.width()));
    uint32_t y = static_cast<uint32_t>(floor(hilbertMax * ((r.minY + r.maxY) / 2 - extent.minY) / extent.height()));
    uint32_t v = hilbert(x, y);
    return v;
}

const uint32_t hilbertMax = (1 << 16) - 1;

void hilbertSort(std::vector<Item *> &items)
{
    Rect extent = std::accumulate(items.begin(), items.end(), Rect::createInvertedInfiniteRect(), [] (Rect a, Item *b) {
        a.expand(b->rect);
        return a;
    });
    std::sort(items.begin(), items.end(), [&extent] (Item *a, Item *b) {
        uint32_t ha = hilbert(a->rect, hilbertMax, extent);
        uint32_t hb = hilbert(b->rect, hilbertMax, extent);
        return ha > hb;
    });
}

Rect calcExtent(std::vector<Rect> &rects)
{
    Rect extent = std::accumulate(rects.begin(), rects.end(), Rect::createInvertedInfiniteRect(), [] (Rect a, Rect b) {
        a.expand(b);
        return a;
    });
    return extent;
}

Rect calcExtent(std::vector<Item *> &rectitems)
{
    Rect extent = std::accumulate(rectitems.begin(), rectitems.end(), Rect::createInvertedInfiniteRect(), [] (Rect a, Item *b) {
        a.expand(b->rect);
        return a;
    });
    return extent;
}

void hilbertSort(std::vector<Rect> &items)
{
    Rect extent = calcExtent(items);
    std::sort(items.begin(), items.end(), [&extent] (Rect a, Rect b) {
        uint32_t ha = hilbert(a, hilbertMax, extent);
        uint32_t hb = hilbert(b, hilbertMax, extent);
        return ha > hb;
    });
}

void PackedRTree::init(const uint16_t nodeSize)
{
    if (_numItems == 0)
        throw std::invalid_argument("Cannot create empty tree");

    _nodeSize = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));

    uint64_t n = _numItems;
    uint64_t numNodes = n;
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

void PackedRTree::generateNodes()
{
    for (uint64_t i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
        uint64_t end = _levelBounds[i];
        while (pos < end) {
            Rect nodeRect = Rect::createInvertedInfiniteRect();
            uint64_t nodeIndex = pos;
            for (uint64_t j = 0; j < _nodeSize && pos < end; j++)
                nodeRect.expand(_rects[pos++]);
            _rects.push_back(nodeRect);
            _indices.push_back(nodeIndex);
        }
    }
}

void PackedRTree::fromData(const void *data)
{
    auto buf = reinterpret_cast<const uint8_t *>(data);
    const Rect *pr = reinterpret_cast<const Rect*>(buf);
    for (uint64_t i = 0; i < _numNodes; i++) {
        Rect r = *pr++;
        _rects.push_back(r);
        _extent.expand(r);
    }
    uint64_t rectsSize = _numNodes * sizeof(Rect);
    const uint32_t *pi = reinterpret_cast<const uint32_t*>(buf + rectsSize);
    for (uint32_t i = 0; i < _numNonLeafNodes; i++)
        _indices[i] = *pi++;
}

static std::vector<Rect> convert(std::vector<Item *> &items)
{
    std::vector<Rect> rects;
    for (const Item *item: items)
        rects.push_back(item->rect);
    return rects;
}

PackedRTree::PackedRTree(std::vector<Item *> &items, Rect extent, const uint16_t nodeSize) :
    _extent(extent),
    _rects(convert(items)),
    _numItems(_rects.size())
{
    init(nodeSize);
    generateNodes();
}

PackedRTree::PackedRTree(std::vector<Rect> &rects, Rect extent, const uint16_t nodeSize) :
    _extent(extent),
    _rects(rects),
    _numItems(_rects.size())
{
    init(nodeSize);
    generateNodes();
}

PackedRTree::PackedRTree(const void *data, const uint64_t numItems, const uint16_t nodeSize) :
    _extent(Rect::createInvertedInfiniteRect()),
    _numItems(numItems)
{
    init(nodeSize);
    fromData(data);
}

std::vector<uint64_t> PackedRTree::search(double minX, double minY, double maxX, double maxY) const
{
    Rect r { minX, minY, maxX, maxY };
    std::vector<uint64_t> queue;
    std::vector<uint64_t> results;
    queue.push_back(_rects.size() - 1);
    queue.push_back(_levelBounds.size() - 1);
    while(queue.size() != 0) {
        uint64_t nodeIndex = queue[queue.size() - 2];
        uint64_t level = queue[queue.size() - 1];
        queue.pop_back();
        queue.pop_back();
        // find the end index of the node
        uint64_t end = std::min(static_cast<uint64_t>(nodeIndex + _nodeSize), _levelBounds[level]);
        // search through child nodes
        for (uint64_t pos = nodeIndex; pos < end; pos++) {
            uint64_t index = pos < _numItems ? pos : _indices[pos - _numItems];
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
uint64_t PackedRTree::size() const { return _numNodes * sizeof(Rect) + _numNonLeafNodes * sizeof(uint32_t); }

uint64_t PackedRTree::size(const uint64_t numItems, const uint16_t nodeSize)
{
    const uint16_t nodeSizeMin = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));
    uint64_t n = numItems;
    uint64_t numNodes = n;
    do {
        n = (n + nodeSizeMin - 1) / nodeSizeMin;
        numNodes += n;
    } while (n != 1);
    return numNodes * sizeof(Rect) + (numNodes - numItems) * sizeof(uint32_t);
}

uint8_t *PackedRTree::toData() const {
    uint64_t rectsSize = _numNodes * sizeof(Rect);
    uint64_t indicesSize = _numNonLeafNodes * sizeof(uint64_t);
    uint8_t *data = new uint8_t[rectsSize + indicesSize];
    Rect *pr = reinterpret_cast<Rect *>(data);
    for (uint64_t i = 0; i < _numNodes; i++)
        *pr++ = _rects[i];
    uint32_t *pi = reinterpret_cast<uint32_t *>(data + rectsSize);
    for (uint32_t i = 0; i < _numNonLeafNodes; i++)
        *pi++ = _indices[i];
    return data;
}

Rect PackedRTree::getExtent() const { return _extent; }

}
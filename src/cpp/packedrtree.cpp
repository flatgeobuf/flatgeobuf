#include "packedrtree.h"

namespace FlatGeobuf
{

void Node::expand(const Node& r)
{
    if (r.minX < minX) minX = r.minX;
    if (r.minY < minY) minY = r.minY;
    if (r.maxX > maxX) maxX = r.maxX;
    if (r.maxY > maxY) maxY = r.maxY;
}

Node Node::create(uint64_t index)
{
    return {
        std::numeric_limits<double>::infinity(),
        std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity(),
        -1 * std::numeric_limits<double>::infinity(),
        index
    };
}

bool Node::intersects(const Node& r) const
{
    if (maxX < r.minX) return false;
    if (maxY < r.minY) return false;
    if (minX > r.maxX) return false;
    if (minY > r.maxY) return false;
    return true;
}

std::vector<double> Node::toVector()
{
    return std::vector<double> { minX, minY, maxX, maxY };
}

// Based on public domain code at https://github.com/rawrunprotected/hilbert_curves
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

uint32_t hilbert(const Node& r, uint32_t hilbertMax, const Node& extent)
{
    uint32_t x = static_cast<uint32_t>(floor(hilbertMax * ((r.minX + r.maxX) / 2 - extent.minX) / extent.width()));
    uint32_t y = static_cast<uint32_t>(floor(hilbertMax * ((r.minY + r.maxY) / 2 - extent.minY) / extent.height()));
    uint32_t v = hilbert(x, y);
    return v;
}

const uint32_t hilbertMax = (1 << 16) - 1;

void hilbertSort(std::vector<std::shared_ptr<Item>> &items)
{
    Node extent = std::accumulate(items.begin(), items.end(), Node::create(0), [] (Node a, std::shared_ptr<Item> b) {
        a.expand(b->node);
        return a;
    });
    std::sort(items.begin(), items.end(), [&extent] (std::shared_ptr<Item> a, std::shared_ptr<Item> b) {
        uint32_t ha = hilbert(a->node, hilbertMax, extent);
        uint32_t hb = hilbert(b->node, hilbertMax, extent);
        return ha > hb;
    });
}

Node calcExtent(const std::vector<Node> &nodes)
{
    Node extent = std::accumulate(nodes.begin(), nodes.end(), Node::create(0), [] (Node a, const Node& b) {
        a.expand(b);
        return a;
    });
    return extent;
}

Node calcExtent(const std::vector<std::shared_ptr<Item>> &items)
{
    Node extent = std::accumulate(items.begin(), items.end(), Node::create(0), [] (Node a, std::shared_ptr<Item> b) {
        a.expand(b->node);
        return a;
    });
    return extent;
}

void hilbertSort(std::vector<Node> &items)
{
    Node extent = calcExtent(items);
    std::sort(items.begin(), items.end(), [&extent] (const Node& a, const Node& b) {
        uint32_t ha = hilbert(a, hilbertMax, extent);
        uint32_t hb = hilbert(b, hilbertMax, extent);
        return ha > hb;
    });
}

void PackedRTree::init(const uint16_t nodeSize)
{
    if (nodeSize < 2)
        throw std::invalid_argument("Node size must be at least 2");
    if (_numItems == 0)
        throw std::invalid_argument("Cannot create empty tree");
    _nodeSize = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));
    _levelBounds = generateLevelBounds(_numItems, _nodeSize);
    _numNodes = _levelBounds.back();
    _nodes.reserve(static_cast<size_t>(_numNodes));
}

std::vector<uint64_t> PackedRTree::generateLevelBounds(const uint64_t numItems, const uint16_t nodeSize) {
    if (nodeSize < 2)
        throw std::invalid_argument("Node size must be at least 2");
    if (numItems == 0)
        throw std::invalid_argument("Number of items must be greater than 0");
    if (numItems > std::numeric_limits<uint64_t>::max() - ((numItems / nodeSize) * 2))
        throw std::overflow_error("Number of items too large");
    std::vector<uint64_t> levelBounds;
    uint64_t n = numItems;
    uint64_t numNodes = n;
    levelBounds.push_back(n);
    do {
        n = (n + nodeSize - 1) / nodeSize;
        numNodes += n;
        levelBounds.push_back(numNodes);
    } while (n != 1);
    return levelBounds;
}

void PackedRTree::generateNodes()
{
    for (uint32_t i = 0, pos = 0; i < _levelBounds.size() - 1; i++) {
        uint32_t end = static_cast<uint32_t>(_levelBounds[i]);
        while (pos < end) {
            Node node = Node::create(pos);
            for (uint32_t j = 0; j < _nodeSize && pos < end; j++)
                node.expand(_nodes[pos++]);
            _nodes.push_back(node);
        }
    }
}

void PackedRTree::fromData(const void *data)
{
    auto buf = reinterpret_cast<const uint8_t *>(data);
    const Node *pn = reinterpret_cast<const Node *>(buf);
    for (uint64_t i = 0; i < _numNodes; i++) {
        Node n = *pn++;
        _nodes.push_back(n);
        _extent.expand(n);
    }
}

static std::vector<Node> convert(const std::vector<std::shared_ptr<Item>> &items)
{
    std::vector<Node> nodes;
    for (const std::shared_ptr<Item> item: items)
        nodes.push_back(item->node);
    return nodes;
}

PackedRTree::PackedRTree(const std::vector<std::shared_ptr<Item>> &items, const Node& extent, const uint16_t nodeSize) :
    _extent(extent),
    _nodes(convert(items)),
    _numItems(items.size())
{
    init(nodeSize);
    generateNodes();
}

PackedRTree::PackedRTree(const std::vector<Node> &nodes, const Node& extent, const uint16_t nodeSize) :
    _extent(extent),
    _nodes(nodes),
    _numItems(nodes.size())
{
    init(nodeSize);
    generateNodes();
}

PackedRTree::PackedRTree(const void *data, const uint64_t numItems, const uint16_t nodeSize) :
    _extent(Node::create(0)),
    _numItems(numItems)
{
    init(nodeSize);
    fromData(data);
}

std::vector<uint64_t> PackedRTree::search(double minX, double minY, double maxX, double maxY) const
{
    Node n { minX, minY, maxX, maxY };
    std::vector<uint64_t> queue;
    std::vector<uint64_t> results;
    queue.push_back(_nodes.size() - 1);
    queue.push_back(_levelBounds.size() - 1);
    while(queue.size() != 0) {
        uint64_t nodeIndex = queue[queue.size() - 2];
        uint64_t level = queue[queue.size() - 1];
        queue.pop_back();
        queue.pop_back();
        // find the end index of the node
        uint64_t end = std::min(static_cast<uint64_t>(nodeIndex + _nodeSize), _levelBounds[static_cast<size_t>(level)]);
        // search through child nodes
        for (uint64_t pos = nodeIndex; pos < end; pos++) {
            auto node = _nodes[static_cast<size_t>(pos)];
            if (!n.intersects(node))
                continue;
            if (nodeIndex < _numItems) {
                results.push_back(pos); // leaf item
            } else {
                queue.push_back(node.index); // node; add it to the search queue
                queue.push_back(level - 1);
            }
        }
    }
    return results;
}

std::vector<uint64_t> PackedRTree::streamSearch(
    const uint64_t numItems, const uint16_t nodeSize, const Node& n,
    const std::function<void(uint8_t *, size_t, size_t)> &readNode)
{
    auto levelBounds = generateLevelBounds(numItems, nodeSize);
    uint64_t numNodes = levelBounds.back();
    std::vector<Node> nodes;
    nodes.reserve(nodeSize);
    uint8_t *nodesBuf = reinterpret_cast<uint8_t *>(nodes.data());
    std::vector<uint64_t> queue;
    std::vector<uint64_t> results;
    queue.push_back(numNodes - 1);
    queue.push_back(levelBounds.size() - 1);
    while(queue.size() != 0) {
        uint64_t nodeIndex = queue[queue.size() - 2];
        bool isLeafNode = nodeIndex < numItems;
        uint64_t level = queue[queue.size() - 1];
        queue.pop_back();
        queue.pop_back();
        // find the end index of the node
        uint64_t end = std::min(static_cast<uint64_t>(nodeIndex + nodeSize), levelBounds[static_cast<size_t>(level)]);
        uint64_t length = end - nodeIndex;
        readNode(nodesBuf, static_cast<size_t>(nodeIndex * sizeof(Node)), static_cast<size_t>(length * sizeof(Node)));
        // search through child nodes
        for (uint64_t pos = nodeIndex; pos < end; pos++) {
            uint64_t nodePos = pos - nodeIndex;
            auto node = nodes[static_cast<size_t>(nodePos)];
            if (!n.intersects(node))
                continue;
            if (isLeafNode) {
                results.push_back(pos); // leaf item
            } else {
                queue.push_back(node.index); // node; add it to the search queue
                queue.push_back(level - 1);
            }
        }
    }
    return results;
}

uint64_t PackedRTree::size() const { return _numNodes * sizeof(Node); }

uint64_t PackedRTree::size(const uint64_t numItems, const uint16_t nodeSize)
{
    if (nodeSize < 2)
        throw std::invalid_argument("Node size must be at least 2");
    if (numItems == 0)
        throw std::invalid_argument("Number of items must be greater than 0");
    const uint16_t nodeSizeMin = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));
    // limit so that resulting size in bytes can be represented by uint64_t
    if (numItems > static_cast<uint64_t>(1) << 56)
        throw std::overflow_error("Number of items must be less than 2^56");
    uint64_t n = numItems;
    uint64_t numNodes = n;
    do {
        n = (n + nodeSizeMin - 1) / nodeSizeMin;
        numNodes += n;
    } while (n != 1);
    return numNodes * sizeof(Node);
}

void PackedRTree::streamWrite(const std::function<void(uint8_t *, size_t)> &writeData) {
    writeData(reinterpret_cast<uint8_t *>(_nodes.data()), _nodes.size() * sizeof(Node));
}

Node PackedRTree::getExtent() const { return _extent; }

}
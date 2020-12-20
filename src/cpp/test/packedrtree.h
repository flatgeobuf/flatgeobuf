#include <random>

#include "catch.hpp"

#include "../packedrtree.h"

using namespace FlatGeobuf;

TEST_CASE("PackedRTree")
{
    SECTION("PackedRTree 2 item one dimension")
    {
        std::vector<NodeItem> nodes;
        nodes.push_back({0, 0, 0, 0});
        nodes.push_back({0, 0, 0, 0});
        NodeItem extent = calcExtent(nodes);
        REQUIRE(nodes[0].intersects({0, 0, 0, 0}) == true);
        hilbertSort(nodes);
        uint64_t offset = 0;
        for (auto &node : nodes)
            node.offset = offset += sizeof(NodeItem);
        REQUIRE(nodes[0].intersects({0, 0, 0, 0}) == true);
        PackedRTree tree(nodes, extent);
        auto list = tree.search(0, 0, 0, 0);
        REQUIRE(list.size() == 2);
        REQUIRE(nodes[list[0].index].intersects({0, 0, 0, 0}) == true);
    }

    SECTION("PackedRTree 2 items 2")
    {
        std::vector<NodeItem> nodes;
        nodes.push_back({0, 0, 1, 1});
        nodes.push_back({2, 2, 3, 3});
        NodeItem extent = calcExtent(nodes);
        REQUIRE(nodes[0].intersects({0, 0, 1, 1}) == true);
        REQUIRE(nodes[1].intersects({2, 2, 3, 3}) == true);
        hilbertSort(nodes);
        uint64_t offset = 0;
        for (auto &node : nodes) {
            node.offset = offset;
            offset += sizeof(NodeItem);
        }
        REQUIRE(nodes[1].intersects({0, 0, 1, 1}) == true);
        REQUIRE(nodes[0].intersects({2, 2, 3, 3}) == true);
        PackedRTree tree(nodes, extent);
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
        REQUIRE(nodes[list[0].index].intersects({0, 0, 1, 1}) == true);
    }

    SECTION("PackedRTree 2 rectitems 2")
    {
        std::vector<std::shared_ptr<Item>> items;
        auto r1 = std::make_shared<Item>();
        auto r2 = std::make_shared<Item>();
        r1->nodeItem = {0, 0, 1, 1};
        r2->nodeItem = {2, 2, 3, 3};
        items.push_back(r1);
        items.push_back(r2);
        NodeItem extent = calcExtent(items);
        REQUIRE(items[0]->nodeItem.intersects({0, 0, 1, 1}) == true);
        REQUIRE(items[1]->nodeItem.intersects({2, 2, 3, 3}) == true);
        hilbertSort(items);
        uint64_t offset = 0;
        for (auto &item : items) {
            item->nodeItem.offset = offset;
            offset += sizeof(NodeItem);
        }
        REQUIRE(items[1]->nodeItem.intersects({0, 0, 1, 1}) == true);
        REQUIRE(items[0]->nodeItem.intersects({2, 2, 3, 3}) == true);
        PackedRTree tree(items, extent);
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
        REQUIRE(items[list[0].index]->nodeItem.intersects({0, 0, 1, 1}) == true);
    }

    SECTION("PackedRTree 19 items + roundtrip + streamSearch")
    {
        std::vector<NodeItem> nodes;
        nodes.push_back({0, 0, 1, 1});
        nodes.push_back({2, 2, 3, 3});
        nodes.push_back({10, 10, 11, 11});
        nodes.push_back({100, 100, 110, 110});
        nodes.push_back({101, 101, 111, 111});
        nodes.push_back({102, 102, 112, 112});
        nodes.push_back({103, 103, 113, 113});
        nodes.push_back({104, 104, 114, 114});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        nodes.push_back({10010, 10010, 10110, 10110});
        NodeItem extent = calcExtent(nodes);
        hilbertSort(nodes);
        uint64_t offset = 0;
        for (auto &node : nodes) {
            node.offset = offset;
            offset += sizeof(NodeItem);
        }
        PackedRTree tree(nodes, extent);
        auto list = tree.search(102, 102, 103, 103);
        REQUIRE(list.size() == 4);
        for (uint32_t i = 0; i < list.size(); i++) {
            REQUIRE(nodes[list[i].index].intersects({102, 102, 103, 103}) == true);
        }
        std::vector<uint8_t> treeData;
        tree.streamWrite([&treeData] (uint8_t *buf, size_t size) { std::copy(buf, buf+size, std::back_inserter(treeData)); });
        auto data = treeData.data();

        PackedRTree tree2(data, nodes.size());
        auto list2 = tree2.search(102, 102, 103, 103);
        REQUIRE(list2.size() == 4);
        for (uint32_t i = 0; i < list2.size(); i++) {
            REQUIRE(nodes[list2[i].index].intersects({102, 102, 103, 103}) == true);
        }
        auto readNode = [data] (uint8_t *buf, uint32_t i, uint32_t s) {
            //std::cout << "i: " << i << std::endl;
            std::copy(data + i, data + i + s, buf);
        };
        auto list3 = PackedRTree::streamSearch(nodes.size(), 16, {102, 102, 103, 103}, readNode);
        REQUIRE(list3.size() == 4);
        for (uint32_t i = 0; i < list3.size(); i++) {
            REQUIRE(nodes[list3[i].index].intersects({102, 102, 103, 103}) == true);
        }
    }
    
    SECTION("PackedRTree 100 000 items in denmark")
    {
        std::uniform_real_distribution<double> unifx(466379,708929);
        std::uniform_real_distribution<double> unify(6096801,6322352);
        std::default_random_engine re;
        std::vector<NodeItem> nodes;
        double x, y;
        for (int i = 0; i < 100000; i++) {
            x = unifx(re);
            y = unify(re);
            nodes.push_back({x, y, x, y});
        }
        NodeItem extent = calcExtent(nodes);
        hilbertSort(nodes);
        PackedRTree tree(nodes, extent);
        auto list = tree.search(690407, 6063692, 811682, 6176467);
        for (uint64_t i = 0; i < list.size(); i++)
            CHECK(nodes[list[i].index].intersects({690407, 6063692, 811682, 6176467}) == true);
        
        std::vector<uint8_t> treeData;
        tree.streamWrite([&treeData] (uint8_t *buf, size_t size) { std::copy(buf, buf+size, std::back_inserter(treeData)); });
        auto data = treeData.data();

        auto readNode = [data] (uint8_t *buf, uint32_t i, uint32_t s) {
            //std::cout << "i: " << i << std::endl;
            std::copy(data + i, data + i + s, buf);
        };
        auto list2 = PackedRTree::streamSearch(nodes.size(), 16, {690407, 6063692, 811682, 6176467}, readNode);
        for (uint64_t i = 0; i < list2.size(); i++)
            CHECK(nodes[list2[i].index].intersects({690407, 6063692, 811682, 6176467}) == true);
    }
}
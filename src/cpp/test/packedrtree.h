#include <random>

#include "catch.hpp"

#include "../packedrtree.h"

using namespace FlatGeobuf;

TEST_CASE("PackedRTree")
{
    SECTION("PackedRTree 2 items 2")
    {
        std::vector<Node> nodes;
        nodes.push_back({0, 0, 1, 1});
        nodes.push_back({2, 2, 3, 3});
        Node extent = calcExtent(nodes);
        REQUIRE(nodes[0].intersects({0, 0, 1, 1}) == true);
        REQUIRE(nodes[1].intersects({2, 2, 3, 3}) == true);
        hilbertSort(nodes);
        REQUIRE(nodes[1].intersects({0, 0, 1, 1}) == true);
        REQUIRE(nodes[0].intersects({2, 2, 3, 3}) == true);
        PackedRTree tree(nodes, extent);
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
        REQUIRE(nodes[list[0]].intersects({0, 0, 1, 1}) == true);
    }

    SECTION("PackedRTree 2 rectitems 2")
    {
        std::vector<std::shared_ptr<Item>> items;
        auto r1 = std::make_shared<Item>();
        auto r2 = std::make_shared<Item>();
        r1->node = {0, 0, 1, 1};
        r2->node = {2, 2, 3, 3};
        items.push_back(r1);
        items.push_back(r2);
        Node extent = calcExtent(items);
        REQUIRE(items[0]->node.intersects({0, 0, 1, 1}) == true);
        REQUIRE(items[1]->node.intersects({2, 2, 3, 3}) == true);
        hilbertSort(items);
        REQUIRE(items[1]->node.intersects({0, 0, 1, 1}) == true);
        REQUIRE(items[0]->node.intersects({2, 2, 3, 3}) == true);
        PackedRTree tree(items, extent);
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
        REQUIRE(items[list[0]]->node.intersects({0, 0, 1, 1}) == true);
    }

    /*
    SECTION("PackedHilbertRTree single item")
    {
        PackedHilbertRTree<uint16_t> tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(tree.size() == 68);
        REQUIRE(list.size() == 1);
        for (uint16_t i = 0; i < list.size(); i++) {
            auto index = tree.getIndex(i);
            auto rect = tree.getRect(index);
            REQUIRE(rect.intersects({0, 0, 1, 1}) == true);
        }
    }
    SECTION("PackedHilbertRTree single item uint32_t")
    {
        PackedHilbertRTree<uint32_t> tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(tree.size() == 72);
        REQUIRE(list.size() == 1);
    }
    SECTION("PackedHilbertRTree 2 items")
    {
        PackedHilbertRTree<uint16_t> tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.finish();
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
    }
    SECTION("PackedHilbertRTree 2 items 2")
    {
        PackedHilbertRTree<uint16_t> tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        REQUIRE(tree.getRect(0).intersects({0, 0, 1, 1}) == true);
        REQUIRE(tree.getRect(1).intersects({2, 2, 3, 3}) == true);
        REQUIRE(tree.getIndex(0) == 0);
        REQUIRE(tree.getIndex(1) == 1);
        tree.finish();
        REQUIRE(tree.getRect(1).intersects({0, 0, 1, 1}) == true);
        REQUIRE(tree.getRect(0).intersects({2, 2, 3, 3}) == true);
        REQUIRE(tree.getIndex(1) == 0);
        REQUIRE(tree.getIndex(0) == 1);
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
        REQUIRE(tree.getIndex(list[0]) == 1);
    }
    SECTION("PackedHilbertRTree roundtrip 1 item")
    {
        PackedHilbertRTree<uint16_t> tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto data = tree.toData();
        auto tree2 = PackedHilbertRTree<uint16_t>(1, 16, data);
        auto list = tree2.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
    }
    SECTION("PackedHilbertRTree roundtrip 2 items")
    {
        PackedHilbertRTree<uint16_t> tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.finish();
        auto data = tree.toData();
        auto tree2 = PackedHilbertRTree<uint16_t>(2, 16, data);
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
    }
    SECTION("PackedHilbertRTree 3 points detailed verification")
    {
        PackedHilbertRTree<uint16_t> tree(3);
        tree.add(0, 0, 0, 0);
        tree.add(1, 1, 1, 1);
        tree.add(2, 2, 2, 2);
        REQUIRE(tree.getRect(0).intersects({0, 0, 0, 0}) == true);
        REQUIRE(tree.getRect(1).intersects({1, 1, 1, 1}) == true);
        REQUIRE(tree.getRect(2).intersects({2, 2, 2, 2}) == true);
        REQUIRE(tree.getIndex(0) == 0);
        REQUIRE(tree.getIndex(1) == 1);
        REQUIRE(tree.getIndex(2) == 2);
        tree.finish();
        REQUIRE(tree.getRect(0).intersects({0, 0, 0, 0}) == true);
        REQUIRE(tree.getRect(2).intersects({1, 1, 1, 1}) == true);
        REQUIRE(tree.getRect(1).intersects({2, 2, 2, 2}) == true);
        REQUIRE(tree.getIndex(0) == 0);
        REQUIRE(tree.getIndex(2) == 1);
        REQUIRE(tree.getIndex(1) == 2);
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
        REQUIRE(list[0] == 2); 
        REQUIRE(list[1] == 1);
        for (uint32_t i = 0; i < list.size(); i++) {
            auto rect = tree.getRect(tree.getIndex(list[i]));
            REQUIRE(rect.intersects({1, 1, 2, 2}) == true);
        }
    }
    SECTION("PackedHilbertRTree 4 items")
    {
        PackedHilbertRTree<uint16_t> tree(4);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(10, 10, 11, 11);
        tree.add(100, 100, 110, 110);
        REQUIRE(tree.getRect(0).intersects({0, 0, 1, 1}) == true);
        REQUIRE(tree.getRect(1).intersects({2, 2, 3, 3}) == true);
        REQUIRE(tree.getRect(2).intersects({10, 10, 11, 11}) == true);
        REQUIRE(tree.getRect(3).intersects({100, 100, 110, 110}) == true);
        REQUIRE(tree.getIndex(0) == 0);
        REQUIRE(tree.getIndex(1) == 1);
        REQUIRE(tree.getIndex(2) == 2);
        REQUIRE(tree.getIndex(3) == 3);
        tree.finish();
        REQUIRE(tree.getRect(0).intersects({0, 0, 1, 1}) == true);
        REQUIRE(tree.getRect(3).intersects({2, 2, 3, 3}) == true);
        REQUIRE(tree.getRect(2).intersects({10, 10, 11, 11}) == true);
        REQUIRE(tree.getRect(1).intersects({100, 100, 110, 110}) == true);
        REQUIRE(tree.getIndex(0) == 0);
        REQUIRE(tree.getIndex(3) == 1);
        REQUIRE(tree.getIndex(2) == 2);
        REQUIRE(tree.getIndex(1) == 3);
        auto list = tree.search(10, 10, 11, 11);
        REQUIRE(list.size() == 1);
        REQUIRE(tree.getIndex(list[0]) == 2);
        REQUIRE(tree.getRect(2).intersects({10, 10, 11, 11}) == true);
    }*/

    SECTION("PackedRTree 19 items + roundtrip + streamSearch")
    {
        std::vector<Node> nodes;
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
        Node extent = calcExtent(nodes);
        hilbertSort(nodes);
        PackedRTree tree(nodes, extent);
        auto list = tree.search(102, 102, 103, 103);
        REQUIRE(list.size() == 4);
        for (uint32_t i = 0; i < list.size(); i++) {
            auto node = nodes[list[i]];
            REQUIRE(node.intersects({102, 102, 103, 103}) == true);
        }
        std::vector<uint8_t> treeData;
        tree.streamWrite([&treeData] (uint8_t *buf, size_t size) { std::copy(buf, buf+size, std::back_inserter(treeData)); });
        auto data = treeData.data();

        PackedRTree tree2(data, nodes.size());
        auto list2 = tree2.search(102, 102, 103, 103);
        REQUIRE(list2.size() == 4);
        for (uint32_t i = 0; i < list2.size(); i++) {
            auto node = nodes[list2[i]];
            REQUIRE(node.intersects({102, 102, 103, 103}) == true);
        }
        auto readNode = [data] (uint8_t *buf, uint32_t i, uint32_t s) {
            std::copy(data + i, data + i + s, buf);
        };
        auto list3 = PackedRTree::streamSearch(nodes.size(), 16, {102, 102, 103, 103}, readNode);
        REQUIRE(list3.size() == 4);
        for (uint32_t i = 0; i < list3.size(); i++) {
            auto node = nodes[list3[i]];
            REQUIRE(node.intersects({102, 102, 103, 103}) == true);
        }
    }
    
    /*SECTION("PackedRTree 1 million items in denmark")
    {
        std::uniform_real_distribution<double> unifx(466379,708929);
        std::uniform_real_distribution<double> unify(6096801,6322352);
        std::default_random_engine re;
        std::vector<Rect> rects;
        double x, y;
        for (int i = 0; i < 1000000; i++) {
            x = unifx(re);
            y = unify(re);
            rects.push_back({x, y, x, y});
        }
        Rect extent = calcExtent(rects);
        hilbertSort(rects);
        PackedRTree tree(rects, extent);
        auto list = tree.search(690407, 6063692, 811682, 6176467);
        for (uint64_t i = 0; i < list.size(); i++) {
            auto rect = rects[list[i]];
            INFO(rect);
            CHECK(rect.intersects({690407, 6063692, 811682, 6176467}) == true);
        }
        auto data = tree.toData();
        auto readNode = [data] (uint8_t *buf, uint32_t i, uint32_t s) {
            std::copy(data + i, data + i + s, buf);
        };
        auto list2 = PackedRTree::streamSearch(rects.size(), 16, {690407, 6063692, 811682, 6176467}, readNode, readNode);
        for (uint64_t i = 0; i < list2.size(); i++) {
            auto rect = rects[list2[i]];
            INFO(rect);
            CHECK(rect.intersects({690407, 6063692, 811682, 6176467}) == true);
        }
    }*/    
}
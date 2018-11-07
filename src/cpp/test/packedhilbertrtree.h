#include <random>

#include "catch.hpp"

#include "../packedhilbertrtree.h"

using namespace FlatGeobuf;

TEST_CASE("PackedHilbertRTree")
{
    SECTION("PackedHilbertRTree single item")
    {
        PackedHilbertRTree<uint16_t> tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(tree.size() == 68);
        REQUIRE(list.size() == 1);
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
    SECTION("PackedHilbertRTree two items")
    {
        PackedHilbertRTree<uint16_t> tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.finish();
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
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
    SECTION("PackedHilbertRTree 4 items")
    {
        PackedHilbertRTree<uint16_t> tree(4);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(10, 10, 11, 11);
        tree.add(100, 100, 110, 110);
        tree.finish();
        auto list = tree.search(10, 10, 11, 11);
        REQUIRE(list.size() == 1);
        REQUIRE(list[0] == 2);
    }
    SECTION("PackedHilbertRTree 8 items")
    {
        PackedHilbertRTree<uint16_t> tree(8);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(10, 10, 11, 11);
        tree.add(100, 100, 110, 110);
        tree.add(101, 101, 111, 111);
        tree.add(102, 102, 112, 112);
        tree.add(103, 103, 113, 113);
        tree.add(104, 104, 114, 114);
        tree.finish();
        auto list = tree.search(10, 10, 100, 100);
        REQUIRE(list.size() == 2);
        REQUIRE(list[0] == 3);
        REQUIRE(list[1] == 2);
    }
    SECTION("PackedHilbertRTree 1 million items")
    {
        std::uniform_real_distribution<double> unif(0,1);
        std::default_random_engine re;
        PackedHilbertRTree<uint32_t> tree(1000000);
        double x, y;
        for (int i = 0; i < 1000000; i++) {
            x = unif(re);
            y = unif(re);
            tree.add(x, y, x, y);
        }
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1000000);
    }
    SECTION("PackedHilbertRTree 3 items replaced root indices")
    {
        PackedHilbertRTree<uint16_t> tree(3);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(4, 4, 5, 5);
        std::vector<uint16_t> rootIndices({80, 60, 70});
        tree.replaceRootIndices(rootIndices);
        tree.finish();
        auto list = tree.search(2, 2, 3, 3);
        REQUIRE(list.size() == 1);
        REQUIRE(list[0] == 60);
    }
}
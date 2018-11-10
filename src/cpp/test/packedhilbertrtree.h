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
    }
    SECTION("PackedHilbertRTree 8 items")
    {
        PackedHilbertRTree<uint32_t> tree(9);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(10, 10, 11, 11);
        tree.add(100, 100, 110, 110);
        tree.add(101, 101, 111, 111);
        tree.add(102, 102, 112, 112);
        tree.add(103, 103, 113, 113);
        tree.add(104, 104, 114, 114);
        tree.add(10010, 10010, 10110, 10110);
        tree.finish();
        auto list = tree.search(102, 102, 103, 103);
        REQUIRE(list.size() == 4);
        for (uint32_t i = 0; i < list.size(); i++) {
            auto rect = tree.getRect(tree.getIndex(list[i]));
            REQUIRE(rect.intersects({102, 102, 103, 103}) == true);
        }
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
    SECTION("PackedHilbertRTree 2 million items")
    {
        std::uniform_real_distribution<double> unif1(0,1);
        std::uniform_real_distribution<double> unif2(100,200);
        std::default_random_engine re;
        PackedHilbertRTree<uint32_t> tree(2000000);
        double x, y;
        for (int i = 0; i < 1000000; i++) {
            x = unif1(re);
            y = unif1(re);
            tree.add(x, y, x, y);
        }
        for (int i = 0; i < 1000000; i++) {
            x = unif2(re);
            y = unif2(re);
            tree.add(x, y, x, y);
        }
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1000000);
        for (uint32_t i = 0; i < list.size(); i++) {
            auto rect = tree.getRect(tree.getIndex(list[i]));
            REQUIRE(rect.intersects({0, 0, 1, 1}) == true);
        }
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
    SECTION("PackedHilbertRTree 1 million items in denmark")
    {
        std::uniform_real_distribution<double> unifx(466379,708929);
        std::uniform_real_distribution<double> unify(6096801,6322352);
        std::default_random_engine re;
        PackedHilbertRTree<uint64_t> tree(1000000);
        double x, y;
        for (int i = 0; i < 1000000; i++) {
            x = unifx(re);
            y = unify(re);
            tree.add(x, y, x, y);
        }
        tree.finish();
        auto list = tree.search(690407, 6063692, 811682, 6176467);
        for (uint64_t i = 0; i < list.size(); i++) {
            auto rect = tree.getRect(tree.getIndex(list[i]));
            INFO(rect);
            REQUIRE(rect.intersects({690407, 6063692, 811682, 6176467}) == true);
        }
    }
}
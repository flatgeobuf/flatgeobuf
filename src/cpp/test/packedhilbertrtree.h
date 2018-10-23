#include <random>

#include "catch.hpp"

#include "../packedhilbertrtree.h"

using namespace FlatGeobuf;

TEST_CASE("PackedHilbertRTree")
{
    SECTION("PackedHilbertRTree single item")
    {
        PackedHilbertRTree tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto list = tree.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
    }
    SECTION("PackedHilbertRTree two items")
    {
        PackedHilbertRTree tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.finish();
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
    }
    SECTION("PackedHilbertRTree roundtrip single item")
    {
        PackedHilbertRTree tree(1);
        tree.add(0, 0, 1, 1);
        tree.finish();
        auto data = tree.toData();
        auto tree2 = PackedHilbertRTree(1, 16, data);
        auto list = tree2.search(0, 0, 1, 1);
        REQUIRE(list.size() == 1);
    }
    SECTION("PackedHilbertRTree roundtrip single item")
    {
        PackedHilbertRTree tree(2);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.finish();
        auto data = tree.toData();
        auto tree2 = PackedHilbertRTree(2, 16, data);
        auto list = tree.search(1, 1, 2, 2);
        REQUIRE(list.size() == 2);
    }
    SECTION("PackedHilbertRTree four items")
    {
        PackedHilbertRTree tree(4);
        tree.add(0, 0, 1, 1);
        tree.add(2, 2, 3, 3);
        tree.add(10, 10, 11, 11);
        tree.add(100, 100, 110, 110);
        tree.finish();
        auto list = tree.search(10, 10, 11, 11);
        REQUIRE(list.size() == 1);
    }
    SECTION("PackedHilbertRTree 1 million items")
    {
        std::uniform_real_distribution<double> unif(0,1);
        std::default_random_engine re;
        PackedHilbertRTree tree(1000000);
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
}
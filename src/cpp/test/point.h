#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../header_generated.h"
#include "../feature_generated.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

TEST_CASE("Point")
{
    SECTION("Verify size of feature/geometry with point")
    {
        int size;

        FlatBufferBuilder fbb1;
        FlatBufferBuilder fbb2;
        FlatBufferBuilder fbb3;

        std::vector<double> xy;
        xy.push_back(0);
        xy.push_back(0);

        auto g1 = CreateGeometryDirect(fbb1, nullptr, nullptr);
        fbb1.Finish(g1);
        size = fbb1.GetSize();
        REQUIRE(size == 12);

        auto g2 = CreateGeometryDirect(fbb2, nullptr, &xy);
        fbb2.Finish(g2);
        size = fbb2.GetSize();
        REQUIRE(size == 40);

        auto g3 = CreateGeometryDirect(fbb3, nullptr, &xy);
        auto f = CreateFeatureDirect(fbb3, g3);
        fbb3.Finish(f);
        size = fbb3.GetSize();
        REQUIRE(size == 56);
    }
}
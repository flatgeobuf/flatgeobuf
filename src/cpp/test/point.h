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
        FlatBufferBuilder fbb;
        FlatBufferBuilder fbb2;

        std::vector<double> xy;
        xy.push_back(0);
        xy.push_back(0);

        auto g = CreateGeometryDirect(fbb, nullptr, &xy);
        fbb.FinishSizePrefixed(g);
        int size = fbb.GetSize();
        REQUIRE(size == 48);

        auto g2 = CreateGeometryDirect(fbb2, nullptr, &xy);
        auto f = CreateFeatureDirect(fbb2, g2);
        fbb.FinishSizePrefixed(f);
        size = fbb2.GetSize();
        REQUIRE(size == 50);
    }
}
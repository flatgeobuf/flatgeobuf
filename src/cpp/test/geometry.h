#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../flatgeobuf_generated.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

TEST_CASE("Geometry")
{
    SECTION("Point")
    {
        FlatBufferBuilder fbb(1024);
        auto envelope = nullptr;
        auto columns = nullptr;
        int features_size = 0;
        int features_count = 1;

        auto header = CreateHeaderDirect(fbb, "Test", envelope, GeometryType::Point, 2, columns, 16, 0, features_size, features_count);

        fbb.FinishSizePrefixed(header);
        //uint8_t *buf = fbb.GetBufferPointer();
        int size = fbb.GetSize();

        REQUIRE(size == 64);

        fbb.Clear();
        auto coords = new std::vector<double>({1, 1});
        auto geometry = CreateGeometryDirect(fbb, nullptr, nullptr, nullptr, nullptr, coords);
        auto feature = CreateFeature(fbb, 0, 0, geometry, 0);
        fbb.FinishSizePrefixed(feature);
        //uint8_t *buf = fbb.GetBufferPointer();
        size = fbb.GetSize();

        REQUIRE(size == 88);
    }
}
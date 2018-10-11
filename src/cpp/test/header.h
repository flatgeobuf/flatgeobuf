#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../flatgeobuf_generated.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

TEST_CASE("Header")
{
    SECTION("Header for empty case")
    {
        FlatBufferBuilder fbb(1024);

        auto envelope = nullptr;
        auto columns = nullptr;
        int features_size = 0;
        int features_count = 0;

        auto header = CreateHeaderDirect(fbb, "Test", envelope, GeometryType::Point, 2, columns, 16, 0, features_size, features_count);

        fbb.FinishSizePrefixed(header);
        uint8_t *buf = fbb.GetBufferPointer();
        int size = fbb.GetSize();

        REQUIRE(size == 36);
    }
}
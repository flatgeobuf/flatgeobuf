#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../header_generated.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

TEST_CASE("Header")
{
    SECTION("Header for empty case")
    {
        FlatBufferBuilder fbb;

        auto envelope = nullptr;
        auto columns = nullptr;
        int features_count = 0;

        auto header = CreateHeaderDirect(fbb, "Test", envelope, GeometryType::Point, false, false, false, false, columns, features_count);

        fbb.FinishSizePrefixed(header);
        uint8_t *buf = fbb.GetBufferPointer();
        int size = fbb.GetSize();

        Verifier v1(buf, size);
        Verifier v2(buf, size);
        Verifier v3(buf + 4, size - 4);
        Verifier v4(buf + 24, size - 24); // intentional move into mid buffer
        REQUIRE(VerifyHeaderBuffer(v1) == true);
        REQUIRE(VerifySizePrefixedHeaderBuffer(v2) == true);
        REQUIRE(VerifyHeaderBuffer(v3) == true);
        REQUIRE(VerifyHeaderBuffer(v4) == false);

        REQUIRE(size == 44);
    }
}
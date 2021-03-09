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

        // create another header and concat to see if the postponed buffer still verifies
        FlatBufferBuilder fbb2;
        std::vector<double> envelope2;
        envelope2.push_back(1.1);
        envelope2.push_back(1.1);
        envelope2.push_back(2.1);
        envelope2.push_back(2.1);
        auto header2 = CreateHeaderDirect(fbb2, "Test2222", &envelope2, GeometryType::Point, false, false, false, false, columns, features_count);
        fbb2.FinishSizePrefixed(header2);
        uint8_t *buf2 = fbb2.GetBufferPointer();
        int size2 = fbb2.GetSize();
        REQUIRE(size2 == 88);

        uint8_t buf3[size + size2];
        memcpy(buf3, buf, size);
        memcpy(buf3 + size, buf2, size2);

        Verifier v5(buf3 + size, size2);
        Verifier v6(buf3 + size, size2);
        REQUIRE(VerifyHeaderBuffer(v5) == true);
        REQUIRE(VerifySizePrefixedHeaderBuffer(v6) == true);
    }
}
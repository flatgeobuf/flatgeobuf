#include <fstream>
#include <sstream>
#include <iostream>

#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../flatgeobuf_generated.h"

#include "../geojson.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

const std::string getFixture(const std::string &path) {
    std::ifstream t(path);
    std::stringstream buffer;
    buffer << t.rdbuf();
    return buffer.str();
}

TEST_CASE("Geometry")
{
    SECTION("Point")
    {
        auto expected = getFixture("src/cpp/test/fixtures/point.geojson");
        std::vector<uint8_t> flatgeobuf;
        serialize(expected, flatgeobuf);
        auto actual = deserialize(flatgeobuf);

        REQUIRE(expected == actual);
        
        /*
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
        */
    }
}
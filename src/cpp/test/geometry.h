#include <fstream>
#include <sstream>
#include <iostream>

#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../flatgeobuf_generated.h"

#include "../geojson.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

const std::string getFixture(const std::string &path) {
    std::ifstream t(path);
    std::stringstream buffer;
    buffer << t.rdbuf();
    return buffer.str();
}

namespace Catch {
    template<>
    struct StringMaker<feature_collection> {
        static std::string convert( feature_collection const& value ) {
            return stringify( value );
        }
    };
}

TEST_CASE("Geometry")
{
    SECTION("Point")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/point.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
}
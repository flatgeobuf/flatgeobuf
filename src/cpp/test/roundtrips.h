#include <fstream>
#include <sstream>
#include <iostream>

#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../header_generated.h"

#include "../geojson.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

const std::string getFixture(const std::string &path)
{
    std::ifstream t(path);
    std::stringstream buffer;
    buffer << t.rdbuf();
    return buffer.str();
}

namespace Catch
{
template<>
struct StringMaker<feature_collection> {
    static std::string convert( feature_collection const& value ) {
        return stringify( value );
    }
};
}

feature_collection roundtrip(feature_collection input) {
    std::vector<uint8_t> flatgeobuf;
    serialize(input, [&flatgeobuf] (uint8_t *data, size_t size) {
        std::copy(data, data + size, std::back_inserter(flatgeobuf));
    });
    auto output = deserialize(flatgeobuf.data());
    return output;
}

TEST_CASE("Geometry roundtrips")
{
    SECTION("Point")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/point.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }

    SECTION("MultiPoint")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multipoint.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }

    SECTION("Points")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/points.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected.size() == actual.size());
    }
    SECTION("LineString")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/linestring.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }
    /*SECTION("MultiLineString")
    {
        auto expected = parse(getFixture("src/cpp/tes t/fixtures/multilinestring.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }*/
    SECTION("MultiLineString single LineString")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multilinestringsingle.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }
    SECTION("Polygon")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/polygon.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }
    SECTION("Polygon with hole")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/polygonwithhole.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }

    /*SECTION("MultiPolygon")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multipolygon.geojson")).get<feature_collection>();
        // Should serialize to 30 flat coords elements, ends [10, 20, 30] and endss [1, 2]
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("MultiPolygon single Polygon")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multipolygonsingle.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }

    SECTION("Bahamas")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/bahamas.geojson")).get<feature_collection>();
        // Should serialize to 42 flat coords elements, ends [16, 28, 42] and endss [1, 1, 1]
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }*/
}

/*TEST_CASE("Attribute roundtrips")
{
    SECTION("Point with properties")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/properties.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }
}*/
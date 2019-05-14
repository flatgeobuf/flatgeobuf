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

TEST_CASE("Geometry")
{
    SECTION("Point")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/point.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }

    SECTION("MultiPoint")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multipoint.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }

    SECTION("Points")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/points.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected.size() == actual.size());
    }
    SECTION("LineString")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/linestring.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("MultiLineString")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multilinestring.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("MultiLineString single LineString")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multilinestringsingle.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("Polygon")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/polygon.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("Polygon with hole")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/polygonwithhole.geojson")).get<feature_collection>();
        auto flatgeobuf = serialize(expected);
        auto actual = deserialize(flatgeobuf);
        REQUIRE(expected == actual);
    }
    SECTION("MultiPolygon")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/multipolygon.geojson")).get<feature_collection>();
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
}
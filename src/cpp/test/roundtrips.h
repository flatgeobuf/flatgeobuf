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

feature_collection roundtrip(feature_collection input, bool createIndex = false) {
    std::vector<uint8_t> flatgeobuf;
    serialize(flatgeobuf, input, createIndex);
    const auto output = deserialize(flatgeobuf.data());
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

    SECTION("Points spatial query")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/points.geojson")).get<feature_collection>();
        std::vector<uint8_t> flatgeobuf;
        serialize(flatgeobuf, expected, true);
        const auto fc1 = deserialize(flatgeobuf.data(), Rect { 0, 0, 1000, 1000 });
        REQUIRE(4 == fc1.size());
        const auto fc2 = deserialize(flatgeobuf.data(), Rect { 0, 0, 1, 1 });
        REQUIRE(1 == fc2.size());
        const auto fc3 = deserialize(flatgeobuf.data(), Rect { 10, 10, 100, 100 });
        REQUIRE(2 == fc3.size());
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
    }*/

    /*SECTION("Bahamas")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/bahamas.geojson")).get<feature_collection>();
        // Should serialize to 42 flat coords elements, ends [16, 28, 42] and endss [1, 1, 1]
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }*/

    SECTION("poly_landmarks")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/poly_landmarks.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }

    /*SECTION("poly_landmarks spatial query")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/poly_landmarks.geojson")).get<feature_collection>();
        std::vector<uint8_t> flatgeobuf;
        serialize(flatgeobuf, expected, true);
        //auto file = fopen("/tmp/poly.fgb", "wb");
        //fwrite(flatgeobuf.data(), 1, flatgeobuf.size(), file);
        //fclose(file);
        //const auto fc1 = deserialize(flatgeobuf.data(), Rect { 0, 0, 1, 1 });
        //REQUIRE(0 == fc1.size());
        const auto fc2 = deserialize(flatgeobuf.data(), Rect { -73.996035, 40.730647, -73.987054,40.738246 });
        //const auto fc2 = deserialize(flatgeobuf.data(), Rect { 40.730647, -73.996035, 40.738246, -73.987054 });
        REQUIRE(0 == fc2.size());
    }*/
}

TEST_CASE("Attribute roundtrips")
{
    SECTION("Point with properties")
    {
        auto expected = parse(getFixture("src/cpp/test/fixtures/properties.geojson")).get<feature_collection>();
        auto actual = roundtrip(expected);
        REQUIRE(expected == actual);
    }
}
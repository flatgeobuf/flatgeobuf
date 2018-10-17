#include <mapbox/geojson.hpp>
#include <mapbox/geojson_impl.hpp>
#include <mapbox/geojson/rapidjson.hpp>
#include <mapbox/geometry.hpp>

#include <cppitertools/chunked.hpp>
#include <cppitertools/imap.hpp>

#include <iostream>
#include <algorithm>
#include <vector>

#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

GeometryType toGeometryType(geometry geometry)
{
    if (geometry.is<point>())
        return GeometryType::Point;
    if (geometry.is<line_string>())
        return GeometryType::LineString;
    if (geometry.is<polygon>())
        return GeometryType::Polygon;
    throw std::invalid_argument("Unknown geometry type");
}

const std::vector<uint8_t> serialize(const feature_collection fc) {
    const auto featuresCount = fc.size();

    if (featuresCount == 0)
        throw std::invalid_argument("Cannot serialize empty feature collection");

    const auto featureFirst = fc.at(0);
    const auto geometryType = toGeometryType(featureFirst.geometry);

    FlatBufferBuilder fbb(1024);
    auto columns = nullptr;
    auto header = CreateHeaderDirect(
        fbb, nullptr, nullptr, geometryType, 2, columns, 16, 0, 0, featuresCount);
    fbb.FinishSizePrefixed(header);
    uint8_t* buf = fbb.GetBufferPointer();
    int size = fbb.GetSize();

    std::vector<uint8_t> flatgeobuf;

    std::copy(buf, buf+size, std::back_inserter(flatgeobuf));

    fbb.Clear();
    auto coords = std::vector<double>();
    for_each_point(featureFirst.geometry, [&coords] (point p) { coords.push_back(p.x); coords.push_back(p.y); });
    auto geometry = CreateGeometryDirect(fbb, nullptr, nullptr, nullptr, nullptr, &coords);
    auto feature = CreateFeatureDirect(fbb, 0, 0, geometry, 0);
    fbb.FinishSizePrefixed(feature);
    buf = fbb.GetBufferPointer();
    size = fbb.GetSize();

    std::copy(buf, buf+size, std::back_inserter(flatgeobuf));

    return flatgeobuf;
}

const std::vector<point> extractPoints(const Vector<double>* coords, u_int32_t length, u_int32_t offset = 0)
{
    auto it = coords->begin() + offset;
    std::vector<double> v { it, it + length };
    auto pairs = iter::chunked(v, 2);
    auto points2 = iter::imap([] (auto pair) { return point( pair[0], pair[1]);}, pairs);
    std::vector<point> points { points2.begin(), points2.end() };
    return points;
}

const std::vector<point> extractPoints(const Vector<double>* coords, u_int32_t offset = 0)
{
    return extractPoints(coords, coords->size(), offset);
}

const geometry fromGeometry(const Geometry* geometry, const GeometryType geometryType)
{
    auto coords = geometry->coords();
    switch (geometryType)
    {
        case GeometryType::Point:
            return point { coords->Get(0), coords->Get(1) };
        case GeometryType::LineString:
            return line_string(extractPoints(coords));
        case GeometryType::Polygon:
            return polygon { linear_ring { extractPoints(coords) } };
        default:
            throw std::invalid_argument("Unknown geometry type");
    }
}

const mapbox::geometry::feature<double> fromFeature(const Feature* feature, const GeometryType geometryType)
{
    auto geometry = feature->geometry();
    mapbox::geometry::feature<double> f { fromGeometry(geometry, geometryType) };
    return f;
}

const feature_collection deserialize(std::vector<uint8_t> flatgeobuf)
{
    auto data = flatgeobuf.data();

    u_int32_t headerSize = *data + 4;

    auto header = GetSizePrefixedHeader(data);
    const auto featuresCount = header->features_count();
    const auto geometryType = header->geometry_type();

    feature_collection fc {};

    for (auto i = 0; i < featuresCount; i++) {
        auto feature = GetSizePrefixedRoot<Feature>(data + headerSize);
        auto f = fromFeature(feature, geometryType);
        fc.push_back(f);
    }
    
    return fc;
}
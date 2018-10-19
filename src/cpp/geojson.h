#include <mapbox/geojson.hpp>
#include <mapbox/geojson_impl.hpp>
#include <mapbox/geojson/rapidjson.hpp>
#include <mapbox/geometry.hpp>
#include <mapbox/geometry/envelope.hpp>

#include <iostream>
#include <algorithm>
#include <vector>

#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"

#include "packedhilbertrtree.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

Rect toRect(geometry geometry)
{
    auto box = envelope(geometry);
    return { box.min.x, box.min.y, box.max.x, box.max.y };
}

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

const u_int8_t* serialize(const feature_collection fc) {
    u_int8_t* buf;

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
    buf = fbb.GetBufferPointer();
    int size = fbb.GetSize();

    std::vector<u_int8_t> flatgeobuf;
    std::copy(buf, buf+size, std::back_inserter(flatgeobuf));

    PackedHilbertRTree tree(featuresCount);
    for (u_int32_t i = 0; i < featuresCount; i++) {
        tree.add(toRect(fc[i].geometry));
    }
    tree.finish();

    auto indices = tree.getIndices();
    std::vector<u_int64_t> featureOffsets;
    u_int64_t featureOffset = 0;
    for (u_int32_t i = 0; i < featuresCount; i++) {
        auto f = fc[indices[i]];
        FlatBufferBuilder fbb(1024);
        std::vector<double> coords;
        for_each_point(f.geometry, [&coords] (auto p) { coords.push_back(p.x); coords.push_back(p.y); });
        auto geometry = CreateGeometryDirect(fbb, nullptr, nullptr, nullptr, nullptr, &coords);
        auto feature = CreateFeatureDirect(fbb, 0, 0, geometry, 0);
        fbb.FinishSizePrefixed(feature);
        buf = fbb.GetBufferPointer();
        size = fbb.GetSize();
        std::copy(buf, buf+size, std::back_inserter(flatgeobuf));
        featureOffsets.push_back(featureOffset);
        featureOffset += size;
    }
    buf = tree.toData();
    size = tree.size();
    std::copy(buf, buf+size, std::back_inserter(flatgeobuf));
    
    buf = new u_int8_t[flatgeobuf.size() + featureOffsets.size() * 8];
    memcpy(buf, flatgeobuf.data(), flatgeobuf.size());
    memcpy(buf + flatgeobuf.size(), featureOffsets.data(), featureOffsets.size() * 8);

    return buf;
}

const std::vector<point> extractPoints(const double* coords, u_int32_t length, u_int32_t offset = 0)
{
    std::vector<point> points;
    for (u_int32_t i = offset; i < length; i += 2)
        points.push_back(point { coords[i], coords[i+1] });
    return points;

    // Functional variant.. ?
    /*
    auto it = coords->begin() + offset;
    std::vector<double> v { it, it + length };
    auto pairs = iter::chunked(v, 2);
    auto points2 = iter::imap([] (auto pair) { return point( pair[0], pair[1]);}, pairs);
    std::vector<point> points { points2.begin(), points2.end() };
    return points;
    */
}

const geometry fromGeometry(const Geometry* geometry, const GeometryType geometryType)
{
    auto coords = geometry->coords()->data();
    auto coordsLength = geometry->coords()->Length();
    switch (geometryType) {
        case GeometryType::Point:
            return point { coords[0], coords[1] };
        case GeometryType::LineString:
            return line_string(extractPoints(coords, coordsLength));
        case GeometryType::Polygon:
            return polygon { linear_ring { extractPoints(coords, coordsLength) } };
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

const feature_collection deserialize(const void* buf)
{
    const u_int8_t* bytes = static_cast<const u_int8_t*>(buf);
    const u_int32_t headerSize = *reinterpret_cast<const u_int8_t*>(bytes) + 4;

    auto header = GetSizePrefixedHeader(buf);
    const auto featuresCount = header->features_count();
    const auto geometryType = header->geometry_type();

    feature_collection fc {};

    u_int64_t offset = headerSize;
    for (auto i = 0; i < featuresCount; i++) {
        const u_int32_t featureSize = *reinterpret_cast<const u_int8_t*>(bytes + offset) + 4;
        auto feature = GetSizePrefixedRoot<Feature>(bytes + offset);
        auto f = fromFeature(feature, geometryType);
        fc.push_back(f);
        offset += featureSize;
    }

    PackedHilbertRTree tree(featuresCount, 16, bytes + offset);
    
    return fc;
}
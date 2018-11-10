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

uint8_t magicbytes[4] = { 0x66, 0x67, 0x62, 0x00 };

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

ColumnType indexStorageType(uint64_t numNodes)
{
    if (numNodes < std::numeric_limits<uint16_t>::max() / 4)
        return ColumnType::UShort;
    else if (numNodes < std::numeric_limits<uint32_t>::max() / 4)
        return ColumnType::UInt;
    else
        return ColumnType::ULong;
}

const uint8_t* serialize(const feature_collection fc)
{
    const auto featuresCount = fc.size();
    if (featuresCount == 0)
        throw std::invalid_argument("Cannot serialize empty feature collection");

    uint8_t* buf;
    std::vector<uint8_t> data;
    std::copy(magicbytes, magicbytes + 4, std::back_inserter(data));

    PackedHilbertRTree<uint64_t> tree(featuresCount);
    for (auto f : fc)
        tree.add(toRect(f.geometry));
    tree.finish();

    const auto extent = tree.getExtent().toVector();
    const auto featureFirst = fc.at(0);
    const auto geometryType = toGeometryType(featureFirst.geometry);

    FlatBufferBuilder fbb;
    auto columns = nullptr;
    auto header = CreateHeaderDirect(
        fbb, nullptr, &extent, geometryType, 2, columns, featuresCount);
    fbb.FinishSizePrefixed(header);
    auto hbuf = fbb.Release();
    std::copy(hbuf.data(), hbuf.data()+hbuf.size(), std::back_inserter(data));
    
    buf = tree.toData();
    auto size = tree.size();
    std::copy(buf, buf+size, std::back_inserter(data));
    
    std::vector<uint8_t> featureData;
    std::vector<uint64_t> featureOffsets;
    uint64_t featureOffset = 0;
    for (uint32_t i = 0; i < featuresCount; i++) {
        auto f = fc[tree.getIndex(i)];
        FlatBufferBuilder fbb;
        std::vector<double> coords;
        for_each_point(f.geometry, [&coords] (auto p) { coords.push_back(p.x); coords.push_back(p.y); });
        auto geometry = CreateGeometryDirect(fbb, nullptr, nullptr, nullptr, nullptr, &coords);
        auto feature = CreateFeatureDirect(fbb, i, geometry, 0);
        fbb.FinishSizePrefixed(feature);
        auto dbuf = fbb.Release();
        std::copy(dbuf.data(), dbuf.data() + dbuf.size(), std::back_inserter(featureData));
        featureOffsets.push_back(featureOffset);
        featureOffset += dbuf.size();
    }

    std::copy(featureOffsets.data(), featureOffsets.data() + featureOffsets.size() * 8, std::back_inserter(data));
    std::copy(featureData.data(), featureData.data() + featureData.size(), std::back_inserter(data));
    
    buf = new uint8_t[data.size()];
    memcpy(buf, data.data(), data.size());

    return buf;
}

const std::vector<point> extractPoints(const double* coords, uint32_t length, uint32_t offset = 0)
{
    std::vector<point> points;
    for (uint32_t i = offset; i < length; i += 2)
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
    const uint8_t* bytes = static_cast<const uint8_t*>(buf);

    if (bytes[0] != magicbytes[0] ||
        bytes[1] != magicbytes[1] ||
        bytes[2] != magicbytes[2] ||
        bytes[3] != magicbytes[3])
        throw new std::invalid_argument("Not a FlatGeobuf file");
    uint64_t offset = 4; 
    
    const uint32_t headerSize = *reinterpret_cast<const uint8_t*>(bytes + offset) + 4;
    auto header = GetSizePrefixedHeader(bytes + offset);
    const auto featuresCount = header->features_count();
    const auto geometryType = header->geometry_type();

    PackedHilbertRTree<uint64_t> tree(featuresCount, 16, bytes + offset);
    offset += tree.size();

    offset += featuresCount * 8;

    feature_collection fc {};
    offset += headerSize;
    for (auto i = 0; i < featuresCount; i++) {
        const uint32_t featureSize = *reinterpret_cast<const uint8_t*>(bytes + offset) + 4;
        auto feature = GetSizePrefixedRoot<Feature>(bytes + offset);
        auto f = fromFeature(feature, geometryType);
        fc.push_back(f);
        offset += featureSize;
    }
    
    return fc;
}
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

#include "packedrtree.h"

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
    if (geometry.is<multi_point>())
        return GeometryType::MultiPoint;
    if (geometry.is<line_string>())
        return GeometryType::LineString;
    if (geometry.is<multi_line_string>())
        return GeometryType::MultiLineString;
    if (geometry.is<polygon>())
        return GeometryType::Polygon;
    if (geometry.is<multi_polygon>())
        return GeometryType::MultiPolygon;
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

    std::vector<Rect> rects;
    for (auto f : fc)
        rects.push_back(toRect(f.geometry));
    Rect extent = calcExtent(rects);
    PackedRTree tree(rects, extent);

    const auto extentVector = extent.toVector();
    const auto featureFirst = fc.at(0);
    const auto geometryType = toGeometryType(featureFirst.geometry);

    FlatBufferBuilder fbb;
    auto columns = nullptr;
    auto header = CreateHeaderDirect(
        fbb, nullptr, &extentVector, geometryType, 2, columns, featuresCount);
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
        auto f = fc[i];
        FlatBufferBuilder fbb;
        std::vector<double> coords;
        std::vector<uint32_t> lengths;
        std::vector<uint32_t> ringLengths;
        std::vector<uint32_t> ringCounts;
        if (f.geometry.is<multi_line_string>()) {
            auto mls = f.geometry.get<multi_line_string>();
            if (mls.size() > 1)
                for (auto ls : mls)
                    lengths.push_back(ls.size()*2);
        } else if (f.geometry.is<polygon>()) {
            auto p = f.geometry.get<polygon>();
            if (p.size() > 1)
                for (auto lr : p)
                    ringLengths.push_back(lr.size()*2);
        }
        else if (f.geometry.is<multi_polygon>()) {
            auto mp = f.geometry.get<multi_polygon>();
            if (mp.size() == 1){
                auto p = mp[0];
                for (auto lr : p)
                    ringLengths.push_back(lr.size()*2);
            } else {
                for (auto p : mp) {
                    uint32_t length = 0;
                    uint32_t ringCount = 0;
                    for (auto lr : p){
                        uint32_t ringLength = lr.size()*2;
                        length += ringLength;
                        ringLengths.push_back(ringLength);
                        ringCount++;
                    }
                    lengths.push_back(length);
                    ringCounts.push_back(ringCount);
                }
            }
        }
        for_each_point(f.geometry, [&coords] (auto p) { coords.push_back(p.x); coords.push_back(p.y); });
        auto pRingCounts = ringCounts.size() == 0 ? nullptr : &ringCounts;
        auto pRingLengths = ringLengths.size() == 0 ? nullptr : &ringLengths;
        auto pLength = lengths.size() == 0 ? nullptr : &lengths;
        auto geometry = CreateGeometryDirect(fbb, pRingCounts, pRingLengths, pLength, &coords);
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
    for (uint32_t i = offset; i < offset + length; i += 2)
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

const multi_line_string fromMultiLineString(
    const double *coords,
    const size_t coordsLength,
    const Vector<uint32_t> *lengths)
{
    if (lengths == nullptr || lengths->size() < 2)
        return multi_line_string { line_string(extractPoints(coords, coordsLength)) };
    std::vector<line_string> lineStrings;
    size_t offset = 0;
    for (size_t i = 0; i < lengths->size(); i++) {
        lineStrings.push_back(line_string(extractPoints(coords, lengths->Get(i), offset)));
        offset += lengths->Get(i);
    }
    return multi_line_string(lineStrings);
}

const polygon fromPolygon(
    const double *coords,
    const size_t coordsLength,
    const Vector<uint32_t> *ringLengths)
{
    if (ringLengths == nullptr || ringLengths->size() < 2)
        return polygon { extractPoints(coords, coordsLength) };
    std::vector<linear_ring> linearRings;
    size_t offset = 0;
    for (size_t i = 0; i < ringLengths->size(); i++) {
        linearRings.push_back(linear_ring(extractPoints(coords, ringLengths->Get(i), offset)));
        offset += ringLengths->Get(i);
    }
    return polygon(linearRings);
}

const multi_polygon fromMultiPolygon(
    const double *coords,
    const size_t coordsLength,
    const Vector<uint32_t> *lengths,
    const Vector<uint32_t> *ringLengths,
    const Vector<uint32_t> *ringCounts)
{
    if (lengths == nullptr || lengths->size() < 2)
        return multi_polygon { fromPolygon(coords, coordsLength, ringLengths) };
    std::vector<polygon> polygons;
    size_t offset = 0;
    for (size_t i = 0; i < lengths->size(); i++) {
        std::vector<linear_ring> linearRings;
        uint32_t ringCount = ringCounts->Get(i);
        size_t roffset = 0;
        for (size_t j=0; j < ringCount; j++) {
            uint32_t ringLength = ringLengths->Get(j+roffset);
            linearRings.push_back(linear_ring(extractPoints(coords, ringLength, offset)));
            offset += ringLength;
            roffset++;
        }
        polygons.push_back(linearRings);
    }
    return multi_polygon(polygons);
}

const geometry fromGeometry(const Geometry* geometry, const GeometryType geometryType)
{
    auto coords = geometry->coords()->data();
    auto coordsLength = geometry->coords()->Length();
    switch (geometryType) {
        case GeometryType::Point:
            return point { coords[0], coords[1] };
        case GeometryType::MultiPoint:
            return multi_point { extractPoints(coords, coordsLength) };
        case GeometryType::LineString:
            return line_string(extractPoints(coords, coordsLength));
        case GeometryType::MultiLineString: 
            return fromMultiLineString(coords, coordsLength, geometry->lengths());
        case GeometryType::Polygon:
            return fromPolygon(coords, coordsLength, geometry->ring_lengths());
        case GeometryType::MultiPolygon:
            return fromMultiPolygon(coords, coordsLength, geometry->lengths(), geometry->ring_lengths(), geometry->ring_counts());
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

    std::vector<Rect> rects;
    PackedRTree tree(bytes + offset, featuresCount);
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
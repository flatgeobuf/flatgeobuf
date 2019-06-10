#include <mapbox/geojson.hpp>
#include <mapbox/geojson_impl.hpp>
#include <mapbox/geojson/rapidjson.hpp>
#include <mapbox/geometry.hpp>
#include <mapbox/geometry/envelope.hpp>

#include <iostream>
#include <algorithm>
#include <vector>

#include "flatbuffers/flatbuffers.h"
#include "header_generated.h"
#include "feature_generated.h"

#include "packedrtree.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

uint8_t magicbytes[] = { 0x66, 0x67, 0x62, 0x00, 0x66, 0x67, 0x62, 0x00 };

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

static const ColumnType toColumnType(value value)
{
    if (value.is<bool>())
        return ColumnType::Bool;
    if (value.is<uint64_t>())
        return ColumnType::ULong;
    if (value.is<int64_t>())
        return ColumnType::Long;
    if (value.is<double>())
        return ColumnType::Double;
    if (value.is<std::string>())
        return ColumnType::String;
    throw std::invalid_argument("Unknown column type");
}

const uint8_t* serialize(const feature_collection fc)
{
    const auto featuresCount = fc.size();
    if (featuresCount == 0)
        throw std::invalid_argument("Cannot serialize empty feature collection");

    uint8_t* buf;
    std::vector<uint8_t> data;
    std::copy(magicbytes, magicbytes + sizeof(magicbytes), std::back_inserter(data));

    std::vector<Rect> rects;
    for (auto f : fc)
        rects.push_back(toRect(f.geometry));
    Rect extent = calcExtent(rects);
    PackedRTree tree(rects, extent);

    const auto extentVector = extent.toVector();
    const auto featureFirst = fc.at(0);
    const auto geometryType = toGeometryType(featureFirst.geometry);

    FlatBufferBuilder fbb;
    std::vector<flatbuffers::Offset<Column>> columns;
    //auto propertiesSize = featureFirst.properties.size();
    for (auto p : featureFirst.properties)
        columns.push_back(CreateColumnDirect(fbb, p.first.c_str(), toColumnType(p.second)));

    auto header = CreateHeaderDirect(
        fbb, nullptr, &extentVector, geometryType, 2, &columns, featuresCount);
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
        std::vector<uint32_t> ends;
        std::vector<uint32_t> endss;
        if (f.geometry.is<multi_line_string>()) {
            auto mls = f.geometry.get<multi_line_string>();
            if (mls.size() > 1)
                for (auto ls : mls)
                    ends.push_back(ls.size()*2);
        } else if (f.geometry.is<polygon>()) {
            auto p = f.geometry.get<polygon>();
            if (p.size() > 1)
                for (auto lr : p)
                    ends.push_back(lr.size()*2);
        }
        else if (f.geometry.is<multi_polygon>()) {
            auto mp = f.geometry.get<multi_polygon>();
            if (mp.size() == 1){
                auto p = mp[0];
                for (auto lr : p)
                    ends.push_back(lr.size()*2);
            } else {
                for (auto p : mp) {
                    uint32_t length = 0;
                    uint32_t ringCount = 0;
                    for (auto lr : p){
                        uint32_t ringLength = lr.size()*2;
                        length += ringLength;
                        ends.push_back(ringLength);
                        ringCount++;
                    }
                    endss.push_back(ringCount);
                }
            }
        }
        for_each_point(f.geometry, [&coords] (auto p) { coords.push_back(p.x); coords.push_back(p.y); });
        auto pEndss = ends.size() == 0 ? nullptr : &endss;
        auto pEnds = ends.size() == 0 ? nullptr : &ends;
        auto feature = CreateFeatureDirect(fbb, i, pEnds, pEndss, &coords, 0);
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
    const Vector<uint32_t> *ends)
{
    if (ends == nullptr || ends->size() < 2)
        return multi_line_string { line_string(extractPoints(coords, coordsLength)) };
    std::vector<line_string> lineStrings;
    size_t offset = 0;
    for (size_t i = 0; i < ends->size(); i++) {
        lineStrings.push_back(line_string(extractPoints(coords, ends->Get(i), offset)));
        offset += ends->Get(i);
    }
    return multi_line_string(lineStrings);
}

const polygon fromPolygon(
    const double *coords,
    const size_t coordsLength,
    const Vector<uint32_t> *ends)
{
    if (ends == nullptr || ends->size() < 2)
        return polygon { extractPoints(coords, coordsLength) };
    std::vector<linear_ring> linearRings;
    size_t offset = 0;
    for (size_t i = 0; i < ends->size(); i++) {
        linearRings.push_back(linear_ring(extractPoints(coords, ends->Get(i), offset)));
        offset += ends->Get(i);
    }
    return polygon(linearRings);
}

const multi_polygon fromMultiPolygon(
    const double *coords,
    const size_t coordsLength,
    const Vector<uint32_t> *ends,
    const Vector<uint32_t> *endss)
{
    if (endss == nullptr || endss->size() < 2)
        return multi_polygon { fromPolygon(coords, coordsLength, ends) };
    std::vector<polygon> polygons;
    size_t offset = 0;
    for (size_t i = 0; i < endss->size(); i++) {
        std::vector<linear_ring> linearRings;
        uint32_t ringCount = endss->Get(i);
        size_t roffset = 0;
        for (size_t j=0; j < ringCount; j++) {
            uint32_t ringLength = ends->Get(j+roffset);
            linearRings.push_back(linear_ring(extractPoints(coords, ringLength, offset)));
            offset += ringLength;
            roffset++;
        }
        polygons.push_back(linearRings);
    }
    return multi_polygon(polygons);
}

const geometry fromGeometry(const Feature* feature, const GeometryType geometryType)
{
    auto coords = feature->coords()->data();
    auto coordsLength = feature->coords()->Length();
    switch (geometryType) {
        case GeometryType::Point:
            return point { coords[0], coords[1] };
        case GeometryType::MultiPoint:
            return multi_point { extractPoints(coords, coordsLength) };
        case GeometryType::LineString:
            return line_string(extractPoints(coords, coordsLength));
        case GeometryType::MultiLineString: 
            return fromMultiLineString(coords, coordsLength, feature->ends());
        case GeometryType::Polygon:
            return fromPolygon(coords, coordsLength, feature->ends());
        case GeometryType::MultiPolygon:
            return fromMultiPolygon(coords, coordsLength, feature->ends(), feature->endss());
        default:
            throw std::invalid_argument("Unknown geometry type");
    }
}

const mapbox::feature::feature<double> fromFeature(const Feature* feature, const GeometryType geometryType)
{
    mapbox::feature::feature<double> f { fromGeometry(feature, geometryType) };
    return f;
}

const feature_collection deserialize(const void* buf)
{
    const uint8_t* bytes = static_cast<const uint8_t*>(buf);

    if (memcmp(bytes, magicbytes, sizeof(magicbytes)))
        throw new std::invalid_argument("Not a FlatGeobuf file");
    uint64_t offset = sizeof(magicbytes);
    
    const uint32_t headerSize = *reinterpret_cast<const uint8_t*>(bytes + offset) + sizeof(uoffset_t);
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
        const uint32_t featureSize = *reinterpret_cast<const uint8_t*>(bytes + offset) + sizeof(uoffset_t);
        auto feature = GetSizePrefixedRoot<Feature>(bytes + offset);
        auto f = fromFeature(feature, geometryType);
        fc.push_back(f);
        offset += featureSize;
    }
    
    return fc;
}
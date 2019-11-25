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

uint8_t magicbytes[] = { 0x66, 0x67, 0x62, 0x02, 0x66, 0x67, 0x62, 0x00 };

struct ColumnMeta {
    uint8_t type;
    std::string name;
    uint16_t index;
};

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
    throw std::invalid_argument("toGeometryType: Unknown geometry type");
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
    throw std::invalid_argument("toColumnType: Unknown column type");
}

const void parseProperties(
        const mapbox::feature::property_map &property_map,
        std::vector<uint8_t> &properties,
        std::unordered_map<std::string, ColumnMeta> columnMetas) {
    for (const auto& kv : property_map) {
        const auto name = kv.first;
        const auto value = kv.second;
        const auto columnMeta = columnMetas.at(name);
        const auto type = (ColumnType) columnMeta.type;
        const auto column_index = columnMeta.index;
        std::copy(reinterpret_cast<const uint8_t *>(&column_index), reinterpret_cast<const uint8_t *>(&column_index + 1), std::back_inserter(properties));
        if (type == ColumnType::Long) {
            auto val = value.get<std::int64_t>();
            std::copy(reinterpret_cast<const uint8_t *>(&val), reinterpret_cast<const uint8_t *>(&val + 1), std::back_inserter(properties));
        } else if (type == ColumnType::ULong) {
            auto val = value.get<std::uint64_t>();
            std::copy(reinterpret_cast<const uint8_t *>(&val), reinterpret_cast<const uint8_t *>(&val + 1), std::back_inserter(properties));
        } else if (type == ColumnType::Double) {
            auto val = value.get<double>();
            std::copy(reinterpret_cast<const uint8_t *>(&val), reinterpret_cast<const uint8_t *>(&val + 1), std::back_inserter(properties));
        } else if (type == ColumnType::String) {
            const std::string str = value.get<std::string>();
            if (str.length() >= std::numeric_limits<uint32_t>::max())
                throw std::invalid_argument("String too long");
            uint32_t len = static_cast<uint32_t>(str.length());
            std::copy(reinterpret_cast<const uint8_t *>(&len), reinterpret_cast<const uint8_t *>(&len + 1), std::back_inserter(properties));
            std::copy(str.begin(), str.end(), std::back_inserter(properties));
        } else {
            throw std::invalid_argument("parseProperties: Unknown property type");
        }
    }
}

const uint8_t *serialize(const feature_collection fc)
{
    const auto featuresCount = fc.size();
    if (featuresCount == 0)
        throw std::invalid_argument("Cannot serialize empty feature collection");

    uint8_t *buf;
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

    std::unordered_map<std::string, ColumnMeta> columnMetas;
    uint16_t i = 0;
    for (auto p : featureFirst.properties) {
        auto name = p.first;
        auto value = p.second;
        auto type = toColumnType(value);
        columnMetas.insert({ name, ColumnMeta { static_cast<uint8_t>(type), name, i++ } });
        columns.push_back(CreateColumnDirect(fbb, name.c_str(), type));
    }

    auto header = CreateHeaderDirect(
        fbb, nullptr, &extentVector, geometryType, false, false, false, false, &columns, featuresCount);
    fbb.FinishSizePrefixed(header);
    auto hbuf = fbb.Release();
    std::copy(hbuf.data(), hbuf.data()+hbuf.size(), std::back_inserter(data));

    tree.streamWrite([&data] (uint8_t *buf, size_t size) { std::copy(buf, buf+size, std::back_inserter(data)); });

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
            uint32_t end = 0;
            auto mls = f.geometry.get<multi_line_string>();
            if (mls.size() > 1)
                for (auto ls : mls)
                    ends.push_back(end += ls.size());
        } else if (f.geometry.is<polygon>()) {
            uint32_t end = 0;
            auto p = f.geometry.get<polygon>();
            if (p.size() > 1)
                for (auto lr : p)
                    ends.push_back(end += lr.size());
        } else if (f.geometry.is<multi_polygon>()) {
            // TODO: need rework!
            /*
            uint32_t end = 0;
            auto mp = f.geometry.get<multi_polygon>();
            if (mp.size() == 1) {
                auto p = mp[0];
                if (p.size() > 1)
                    for (auto lr : p)
                        ends.push_back(end += lr.size());
            } else {
                for (auto p : mp) {
                    uint32_t ringCount = 0;
                    for (auto lr : p){
                        ends.push_back(end += lr.size());
                        ringCount++;
                    }
                    endss.push_back(ringCount);
                }
            }*/
        }
        for_each_point(f.geometry, [&coords] (auto p) { coords.push_back(p.x); coords.push_back(p.y); });
        auto pEnds = ends.size() == 0 ? nullptr : &ends;
        std::vector<uint8_t> properties;
        parseProperties(f.properties, properties, columnMetas);
        auto pProperties = properties.size() == 0 ? nullptr : &properties;
        auto geometry = CreateGeometryDirect(fbb, pEnds, &coords, nullptr, nullptr, nullptr, nullptr);
        auto feature = CreateFeatureDirect(fbb, geometry, pProperties);
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

const std::vector<point> extractPoints(const double *coords, uint32_t length, uint32_t offset = 0)
{
    std::vector<point> points;
    for (uint32_t i = offset; i < offset + length; i += 2)
        points.push_back(point { coords[i], coords[i + 1] });
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
    const uint32_t coordsLength,
    const Vector<uint32_t> *ends)
{
    if (ends == nullptr || ends->size() < 2)
        return multi_line_string { line_string(extractPoints(coords, coordsLength)) };
    std::vector<line_string> lineStrings;
    uint32_t offset = 0;
    for (uint32_t i = 0; i < ends->size(); i++) {
        uint32_t end = ends->Get(i) << 1;
        lineStrings.push_back(line_string(extractPoints(coords, end - offset, offset)));
        offset = end;
    }
    return multi_line_string(lineStrings);
}

const polygon fromPolygon(
    const double *coords,
    const uint32_t coordsLength,
    const Vector<uint32_t> *ends)
{
    if (ends == nullptr || ends->size() < 2)
        return polygon { extractPoints(coords, coordsLength) };
    std::vector<linear_ring> linearRings;
    uint32_t offset = 0;
    for (uint32_t i = 0; i < ends->size(); i++) {
        uint32_t end = ends->Get(i) << 1;
        linearRings.push_back(linear_ring(extractPoints(coords, end - offset, offset)));
        offset = end;
    }
    return polygon(linearRings);
}

const geometry fromGeometry(const Geometry *geometry, const GeometryType geometryType);

const multi_polygon fromMultiPolygon(const Geometry *geometry) {
    auto parts = geometry->parts();
    auto partsLength = parts->Length();
    std::vector<polygon> polygons;
    for (auto i = 0; i < partsLength; i++) {
        auto part = parts->Get(i);
        auto p = fromGeometry(part, GeometryType::Polygon).get<polygon>();
        polygons.push_back(p);
    }
    return multi_polygon(polygons);
}

static bool isCollection(const GeometryType geometryType) {
    switch (geometryType) {
        case GeometryType::Point:
        case GeometryType::MultiPoint:
        case GeometryType::LineString:
        case GeometryType::MultiLineString: 
        case GeometryType::Polygon:
            return false;
        case GeometryType::MultiPolygon:
        case GeometryType::GeometryCollection:
            return true;
        default:
            throw std::invalid_argument("isCollection: Unknown geometry type");
    }
}

const geometry fromGeometry(const Geometry *geometry, const GeometryType geometryType)
{
    if (!isCollection(geometryType)) {
        auto xy = geometry->xy()->data();
        auto xyLength = geometry->xy()->Length();
        switch (geometryType) {
            case GeometryType::Point:
                return point { xy[0], xy[1] };
            case GeometryType::MultiPoint:
                return multi_point { extractPoints(xy, xyLength) };
            case GeometryType::LineString:
                return line_string(extractPoints(xy, xyLength));
            case GeometryType::MultiLineString: 
                return fromMultiLineString(xy, xyLength, geometry->ends());
            case GeometryType::Polygon:
                return fromPolygon(xy, xyLength, geometry->ends());
            default:
                throw std::invalid_argument("fromGeometry: Unknown geometry type");
        }
    }

    switch (geometryType) {
        case GeometryType::MultiPolygon:
            return fromMultiPolygon(geometry);
        default:
            throw std::invalid_argument("fromGeometry: Unknown geometry type");
    }
}

mapbox::feature::property_map readGeoJsonProperties(const Feature *feature, std::vector<ColumnMeta> columnMetas) {
    auto properties = feature->properties();
    auto property_map = mapbox::feature::property_map();

    if (properties == nullptr)
        return property_map;

    auto data = properties->data();
    auto size = properties->size();

    uoffset_t offset = 0;
    while (offset < size) {
        uint16_t i = *(reinterpret_cast<const uint16_t *>(data + offset));
        offset += sizeof(uint16_t);
        auto column = columnMetas[i];
        auto type = static_cast<ColumnType>(column.type);
        mapbox::feature::value value;
        switch (type) {
            case ColumnType::Long:
                value.set<int64_t>(*(reinterpret_cast<const int64_t *>(data + offset)));
                offset += sizeof(int64_t);
                break;
            case ColumnType::ULong:
                value.set<uint64_t>(*(reinterpret_cast<const uint64_t *>(data + offset)));
                offset += sizeof(uint64_t);
                break;
            case ColumnType::Double:
                value.set<double>(*(reinterpret_cast<const double *>(data + offset)));
                offset += sizeof(double);
                break;
            case ColumnType::String: {
                uint32_t len = *(reinterpret_cast<const uint32_t *>(data + offset));
                offset += sizeof(uint32_t);
                value.set<std::string>(std::string(reinterpret_cast<const char *>(data + offset), len));
                offset += len;
                break;
            }
            default:
                throw std::invalid_argument("Unknown column type");
        }
        property_map.insert({ column.name, value });
    }
    return property_map;
}

const mapbox::feature::feature<double> fromFeature(
    const Feature *feature,
    const GeometryType geometryType,
    std::vector<ColumnMeta> columnMetas)
{
    auto geometry = feature->geometry();
    auto mapboxGeometry = fromGeometry(geometry, geometryType);
    auto mapboxProperties = readGeoJsonProperties(feature, columnMetas);
    mapbox::feature::feature<double> f { mapboxGeometry, mapboxProperties };
    return f;
}

const feature_collection deserialize(const void* buf)
{
    const uint8_t *bytes = static_cast<const uint8_t*>(buf);

    if (memcmp(bytes, magicbytes, sizeof(magicbytes)))
        throw new std::invalid_argument("Not a FlatGeobuf file");
    uint64_t offset = sizeof(magicbytes);
    
    const uint32_t headerSize = *(bytes + offset) + sizeof(uoffset_t);
    auto header = GetSizePrefixedHeader(bytes + offset);
    const auto featuresCount = header->features_count();
    const auto geometryType = header->geometry_type();

    const auto columns = header->columns();
    std::vector<ColumnMeta> columnMetas;

    if (columns != nullptr) {
        for (uint16_t i = 0; i < columns->Length(); i++) {
            auto column = columns->Get(i);
            auto name = column->name()->str();
            auto type = static_cast<uint8_t>(column->type());
            columnMetas.push_back(ColumnMeta { type, name, i });
        }
    }

    std::vector<Rect> rects;
    PackedRTree tree(bytes + offset, featuresCount);
    offset += tree.size();

    offset += featuresCount * 8;

    feature_collection fc {};
    offset += headerSize;
    for (auto i = 0; i < featuresCount; i++) {
        const uint32_t featureSize = *(bytes + offset) + sizeof(uoffset_t);
        auto feature = GetSizePrefixedRoot<Feature>(bytes + offset);
        auto f = fromFeature(feature, geometryType, columnMetas);
        fc.push_back(f);
        offset += featureSize;
    }
    
    return fc;
}
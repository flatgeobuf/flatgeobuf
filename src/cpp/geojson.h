#include <mapbox/geojson.hpp>
#include <mapbox/geojson_impl.hpp>
#include <mapbox/geojson/rapidjson.hpp>
#include <mapbox/geometry.hpp>
#include <mapbox/geometry/envelope.hpp>
#include <mapbox/feature.hpp>

#include <iostream>
#include <algorithm>
#include <vector>
#include <functional>

#include "flatbuffers/flatbuffers.h"
#include "header_generated.h"
#include "feature_generated.h"

#include "packedrtree.h"

using namespace mapbox::geojson;
using namespace flatbuffers;
using namespace FlatGeobuf;

namespace {

uint8_t magicbytes[] = { 0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00 };

struct ColumnMeta {
    uint8_t type;
    std::string name;
    uint16_t index;
};

struct FeatureItem : Item {
    uoffset_t size;
    uint64_t offset;
};

NodeItem toNodeItem(geometry geometry)
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

const ColumnType toColumnType(value value)
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
        std::unordered_map<std::string, ColumnMeta> &columnMetas)
{
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


const uoffset_t writeFeature(
    const feature &f,
    std::unordered_map<std::string, ColumnMeta> &columnMetas,
    const std::function<void(uint8_t *, size_t)> &writeData)
{
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
    auto size = fbb.GetSize();
    writeData(fbb.GetBufferPointer(), size);
    return size;
}

const void introspectColumnMetas(const feature &f, std::vector<ColumnMeta> &columnMetas)
{
    uint16_t i = 0;
    for (auto p : f.properties) {
        auto name = p.first;
        auto value = p.second;
        auto type = toColumnType(value);
        columnMetas.push_back(ColumnMeta { static_cast<uint8_t>(type), name, i++ });
    }
}

const void writeHeader(
    const char *name,
    std::vector<double> *envelope,
    uint16_t indexNodeSize,
    GeometryType geometryType,
    std::vector<ColumnMeta> &columnMetas,
    uint64_t featuresCount,
    const std::function<void(uint8_t *, size_t)> &writeData)
{
    FlatBufferBuilder fbb;
    auto crs = 0;
    std::vector<flatbuffers::Offset<Column>> columns;
    for (auto i = 0; i < columnMetas.size(); i++)
        columns.push_back(CreateColumnDirect(fbb, columnMetas[i].name.c_str(), (ColumnType) columnMetas[i].type));
    auto pColumns = columns.size() > 0 ? &columns : nullptr;
    auto header = CreateHeaderDirect(
        fbb, nullptr, envelope, geometryType, false, false, false, false, pColumns, featuresCount, indexNodeSize, crs);
    fbb.FinishSizePrefixed(header);
    writeData(fbb.GetBufferPointer(), fbb.GetSize());
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

const multi_polygon fromMultiPolygon(const Geometry *geometry)
{
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

const bool isCollection(const GeometryType geometryType)
{
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

mapbox::feature::property_map readGeoJsonProperties(
    const Feature *feature,
    const std::vector<ColumnMeta> &columnMetas) 
{
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
    const std::vector<ColumnMeta> &columnMetas)
{
    auto geometry = feature->geometry();
    auto mapboxGeometry = fromGeometry(geometry, geometryType);
    auto mapboxProperties = readGeoJsonProperties(feature, columnMetas);
    mapbox::feature::feature<double> f { mapboxGeometry, mapboxProperties };
    return f;
}

const uoffset_t readFeature(
    const std::function<void(const void *, const size_t)> &readData,
    const std::function<void(const feature&)> &writeFeature,
    const GeometryType geometryType,
    const std::vector<ColumnMeta> &columnMetas) 
{
    std::vector<uint8_t> buf;
    buf.reserve(sizeof(uoffset_t));
    readData(buf.data(), sizeof(uoffset_t));
    const auto featureSize = *reinterpret_cast<const uoffset_t*>(buf.data());
    buf.reserve(featureSize);
    readData(buf.data(), featureSize);
    auto feature = GetFeature(buf.data());
    auto f = fromFeature(feature, geometryType, columnMetas);
    writeFeature(f);
    return featureSize;
}

}

namespace FlatGeobuf {

const void serialize(
    const std::function<const feature *()> &readFeature,
    const std::function<const void(void *, const size_t)> &writeData,
    const uint64_t featuresCount = 0,
    const bool createIndex = false)
{
    auto f = readFeature();
    if (f == nullptr)
      throw std::runtime_error("Unable to read a feature (need at least one)");

    const auto geometryType = toGeometryType(f->geometry);
    std::vector<ColumnMeta> columnMetas;
    introspectColumnMetas(*f, columnMetas);
    std::unordered_map<std::string, ColumnMeta> columMetasMap;
    for (auto const &columnMeta : columnMetas)
        columMetasMap[columnMeta.name] = columnMeta;

    // no index is requested write in single pass and return
    if (!createIndex) {
        writeData(magicbytes, sizeof(magicbytes));
        writeHeader(nullptr, nullptr, 0, geometryType, columnMetas, featuresCount, writeData);
        while (f != nullptr) {
            writeFeature(*f, columMetasMap, writeData);
            f = readFeature();
        }
        return;
    }

    // index requested need to write in two passes
    const auto tmpfile = std::tmpfile();
    const auto writeTmpData = [&tmpfile] (const void *data, const size_t size) {
        fwrite(data, size, 1, tmpfile);
    };
    std::vector<std::shared_ptr<Item>> items;
    uint64_t featureOffset = 0;
    while (f != nullptr) {
        auto feature = *f;
        auto size = writeFeature(feature, columMetasMap, writeTmpData);
        const auto item = std::make_shared<FeatureItem>();
        item->nodeItem = toNodeItem(feature.geometry);
        item->size = size;
        item->offset = featureOffset;
        featureOffset += size;
        items.push_back(item);
        f = readFeature();
    }
    fflush(tmpfile);
    std::vector<double> envelope;
    NodeItem extent = calcExtent(items);
    envelope = extent.toVector();
    const auto pEnvelope = envelope.size() > 0 ? &envelope : nullptr;

    writeData(magicbytes, sizeof(magicbytes));
    writeHeader(nullptr, pEnvelope, 16, geometryType, columnMetas, items.size(), writeData);

    hilbertSort(items);
    featureOffset = 0;
    for (auto item : items) {
        auto featureItem = std::static_pointer_cast<FeatureItem>(item);
        featureItem->nodeItem.offset = featureOffset;
        featureOffset += featureItem->size;
    }
    PackedRTree tree(items, extent, 16);
    tree.streamWrite(writeData);

    std::vector<uint8_t> buf;
    for (auto item : items) {
        auto featureItem = std::static_pointer_cast<FeatureItem>(item);
        buf.reserve(featureItem->size);
        if (fseek(tmpfile, featureItem->offset, SEEK_SET) != 0)
            throw std::runtime_error("Failed to seek in file");
        if (fread(buf.data(), featureItem->size, 1, tmpfile) != 1)
            throw std::runtime_error("Failed to read data");
        writeData(buf.data(), featureItem->size);
    }
}

const void serialize(
    const feature_collection &fc,
    const std::function<void(void *, size_t)> &writeData,
    const bool createIndex = false)
{
    size_t i = 0;
    size_t size = fc.size();
    const std::function<const feature *()> readFeature = [&fc, &i, &size] () {
        return i < size ? &(fc[i++]) : nullptr;
    };
    serialize(readFeature, writeData, fc.size(), createIndex);
}

const void serialize(std::vector<uint8_t> &flatgeobuf, const feature_collection &fc, const bool createIndex = false)
{
    const auto writeData = [&flatgeobuf] (const void *data, const size_t size) {
        const auto buf = static_cast<const uint8_t *>(data);
        std::copy(buf, buf + size, std::back_inserter(flatgeobuf));
    };
    serialize(fc, writeData, createIndex);
}

const void deserialize(
    const std::function<void(const void *, const size_t)> &readData,
    const std::function<void(const feature&)> &writeFeature,
    const std::function<void(const size_t)> &seekData = nullptr,
    const NodeItem *nodeItem = nullptr)
{
    std::vector<uint8_t> buf;
    buf.reserve(8);
    readData(buf.data(), sizeof(magicbytes));

    if (memcmp(buf.data(), magicbytes, sizeof(magicbytes)))
        throw new std::invalid_argument("Not a FlatGeobuf file");

    uint64_t offset = sizeof(magicbytes);
    readData(buf.data(), sizeof(uoffset_t));
    offset += sizeof(uoffset_t);
    const auto headerSize = *reinterpret_cast<const uoffset_t*>(buf.data());
    buf.reserve(headerSize);
    readData(buf.data(), headerSize);
    offset += headerSize;
    auto header = GetHeader(buf.data());
    const auto featuresCount = header->features_count();
    const auto geometryType = header->geometry_type();
    const auto indexNodeSize = header->index_node_size();

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

    // check if there is an index
    if (indexNodeSize > 0) {
        if (seekData != nullptr && nodeItem != nullptr) {
            // spatial filter requested, read and use index
            const auto treeOffset = offset;
            const auto readNode = [treeOffset, &seekData, &readData] (uint8_t *buf, size_t i, size_t s) {
                seekData(treeOffset + i);
                readData(buf, s);
            };
            const auto result = PackedRTree::streamSearch(featuresCount, indexNodeSize, *nodeItem, readNode);
            offset += PackedRTree::size(featuresCount, indexNodeSize);
            for (auto item : result) {
                seekData(offset + item.offset);
                readFeature(readData, writeFeature, geometryType, columnMetas);
            }
            return;
        } else {
            // ignore index as no filter was requested
            offset += PackedRTree::size(featuresCount, indexNodeSize);
        }   
    }

    // read full dataset
    for (auto i = 0; i < featuresCount; i++)
        readFeature(readData, writeFeature, geometryType, columnMetas);
}

const feature_collection deserialize(const void *buf)
{
    const uint8_t *data = static_cast<const uint8_t*>(buf);
    uint64_t offset = 0;
    feature_collection fc {};
    const auto readData = [&data, &offset] (const void *buf, const size_t size) {
        memcpy(const_cast<void *>(buf), data + offset, size);
        offset += size;
    };
    const auto writeFeature = [&fc] (const feature &f) {
        fc.push_back(f);
    };
    deserialize(readData, writeFeature);
    return fc;
}

const feature_collection deserialize(const void *buf, const NodeItem rect)
{
    const uint8_t *data = static_cast<const uint8_t*>(buf);
    uint64_t offset = 0;
    feature_collection fc {};
    const auto readData = [&data, &offset] (const void *buf, const size_t size) {
        memcpy(const_cast<void *>(buf), data + offset, size);
        offset += size;
    };
    const auto writeFeature = [&fc] (const feature &f) {
        fc.push_back(f);
    };
    const auto seekData = [&offset] (size_t newoffset) {
        offset = newoffset;
    };
    deserialize(readData, writeFeature, seekData, &rect);
    return fc;
}

}

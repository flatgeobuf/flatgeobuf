#include <iostream>
#include <fstream>

#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"
#include "geojson.h"

using namespace flatbuffers;
using namespace FlatGeobuf;


int main() {
    const char* json = "{\"type\":\"FeatureCollection\",\"features\":[{\"type\":\"Feature\",\"geometry\":{\"type\":\"Point\",\"coordinates\":[1,1]}}]}";

    auto buf = serialize(json);

    return 0;

    /*
    FlatBufferBuilder builder(1024);

    auto name = builder.CreateString("Test");
    
    HeaderBuilder header_builder(builder);
    header_builder.add_name(name);
    header_builder.add_geometry_type(GeometryType::Point);
    auto header = header_builder.Finish();

    builder.FinishSizePrefixed(header);
    uint8_t *buf = builder.GetBufferPointer();
    int size = builder.GetSize();

    std::ofstream outfile;

    outfile.open("point.fgb", std::ios::binary | std::ios::out);
    outfile.write((char *) buf, size);
    outfile.close();

    auto header2 = GetSizePrefixedHeader(buf);

    auto andBack = header2->name()->c_str();

    std::cout << andBack << std::endl;
    */
}
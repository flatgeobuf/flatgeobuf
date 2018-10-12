#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"
#include <iostream>
#include <fstream>

#include <fstream>

using namespace flatbuffers;
using namespace FlatGeobuf;

#include "rapidjson/reader.h"

class HeaderReaderHandler : public BaseReaderHandler {

};

int main() {
    HeaderReaderHandler headerReaderHandler();

    


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
}
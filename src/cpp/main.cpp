#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"
#include <iostream>
#include <fstream>

using namespace flatbuffers;
using namespace FlatGeobuf;

int main() {
    FlatBufferBuilder builder(1024);

    auto name = builder.CreateString("Test");

    LayerBuilder layer_builder(builder);
    layer_builder.add_name(name);
    layer_builder.add_geometry_type(GeometryType::Point);
    auto layer = layer_builder.Finish();
    
    std::vector<Offset<Layer>> layers_vector;
    auto layers = builder.CreateVector(layers_vector);
    
    HeaderBuilder header_builder(builder);
    header_builder.add_layers(layers);
    auto header = header_builder.Finish();

    builder.FinishSizePrefixed(header);
    uint8_t *buf = builder.GetBufferPointer();
    int size = builder.GetSize();

    std::ofstream outfile;

    outfile.open("point.fgb", std::ios::binary | std::ios::out);
    outfile.write((char *) buf, size);
    outfile.close();
}
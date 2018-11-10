#include <iostream>
#include <fstream>

#include "flatbuffers/flatbuffers.h"
#include "flatgeobuf_generated.h"
#include "geojson.h"

using namespace flatbuffers;
using namespace FlatGeobuf;


int main() {
    PackedHilbertRTree<uint16_t> tree(3);
    tree.add(0, 0, 0, 0);
    tree.add(1, 1, 1, 1);
    tree.add(2, 2, 2, 2);
    std::cout << "Indexes: ";
    for (uint32_t i = 0; i < 3; i++) {
        std::cout << tree.getIndex(i) << " ";
    }
    std::cout << std::endl;
    tree.finish();
    std::cout << "Indexes after finish: ";
    for (uint32_t i = 0; i < 3; i++) {
        std::cout << tree.getIndex(i) << " ";
    }
    std::cout << std::endl;
    auto list = tree.search(1, 1, 2, 2);
    std::cout << "Search result indexes: ";
    for (uint32_t i = 0; i < list.size(); i++) {
        std::cout << tree.getIndex(i) << " ";
        //auto rect = tree.getRect(tree.getIndex(list[i]));
    }
    std::cout << std::endl;
}
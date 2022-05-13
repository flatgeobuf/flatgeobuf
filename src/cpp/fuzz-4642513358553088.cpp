#include <iostream>
#include <fstream>

#include "flatbuffers/flatbuffers.h"
#include "header_generated.h"

using namespace flatbuffers;
using namespace FlatGeobuf;

uint8_t magicbytes[] = { 0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00 };

int main() {
    std::ifstream infile;
    infile.open("../../test/data/clusterfuzz-testcase-minimized-fgb_fuzzer-4642513358553088", std::ios::binary | std::ios::in);
    infile.seekg(0, std::ios::end);
    auto length = infile.tellg();
    auto headerLength = (int) length - (int) sizeof(magicbytes);
    infile.seekg(sizeof(magicbytes), std::ios::beg);
    const uint8_t *headerDataSizePrefixed = new uint8_t[headerLength];
    infile.read((char *) headerDataSizePrefixed, headerLength);
    infile.close();
    uint32_t sizePrefix = *((uint32_t *)headerDataSizePrefixed);
    uint8_t *headerData = new uint8_t[sizePrefix];
    memcpy(headerData, headerDataSizePrefixed + sizeof(uint32_t), sizePrefix);
    Verifier v1(headerDataSizePrefixed, sizePrefix + sizeof(uint32_t));
    VerifySizePrefixedHeaderBuffer(v1);
    Verifier v2(headerData, sizePrefix, 64U, 1000000U, false);
    VerifyHeaderBuffer(v2);
}
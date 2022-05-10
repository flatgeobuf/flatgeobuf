#include "catch.hpp"

#include "flatbuffers/flatbuffers.h"
#include "../header_generated.h"
#include "../feature_generated.h"

#include <iostream>
#include <fstream>

using namespace flatbuffers;
using namespace FlatGeobuf;

//uint8_t magicbytes[] = { 0x66, 0x67, 0x62, 0x03, 0x66, 0x67, 0x62, 0x00 };

TEST_CASE("Verifier")
{
    SECTION("Verify size header of poly01.fgb")
    {
        std::ifstream infile;
        infile.open("./test/data/poly00.fgb", std::ios::binary | std::ios::in);
        infile.seekg(0, std::ios::end);
        auto length = infile.tellg();
        REQUIRE(length == 6568);
        auto headerLength = (int) length - (int) sizeof(magicbytes);
        REQUIRE(headerLength == 6560);
        infile.seekg(sizeof(magicbytes), std::ios::beg);
        const uint8_t *headerDataSizePrefixed = new uint8_t[headerLength];
        infile.read((char *) headerDataSizePrefixed, headerLength);
        infile.close();
        uint32_t sizePrefix = *((uint32_t *)headerDataSizePrefixed);
        REQUIRE(sizePrefix == 1316);
        uint8_t *headerData = new uint8_t[sizePrefix];
        memcpy(headerData, headerDataSizePrefixed + sizeof(uint32_t), sizePrefix);
        Verifier v1(headerDataSizePrefixed, sizePrefix + sizeof(uint32_t));
        const auto ok1 = VerifySizePrefixedHeaderBuffer(v1);
        REQUIRE(ok1 == true);
        Verifier v2(headerData, sizePrefix, 64U, 1000000U, false);
        const auto ok2 = VerifyHeaderBuffer(v2);
        REQUIRE(ok2 == true);
    }

    SECTION("Verify size header of poly01.fgb")
    {
        std::ifstream infile;
        infile.open("./test/data/poly01.fgb", std::ios::binary | std::ios::in);
        infile.seekg(0, std::ios::end);
        auto length = infile.tellg();
        REQUIRE(length == 6760);
        auto headerLength = (int) length - (int) sizeof(magicbytes);
        REQUIRE(headerLength == 6752);
        infile.seekg(sizeof(magicbytes), std::ios::beg);
        const uint8_t *headerDataSizePrefixed = new uint8_t[headerLength];
        infile.read((char *) headerDataSizePrefixed, headerLength);
        infile.close();
        uint32_t sizePrefix = *((uint32_t *)headerDataSizePrefixed);
        REQUIRE(sizePrefix == 1508);
        uint8_t *headerData = new uint8_t[sizePrefix];
        memcpy(headerData, headerDataSizePrefixed + sizeof(uint32_t), sizePrefix);
        Verifier v1(headerDataSizePrefixed, sizePrefix + sizeof(uint32_t));
        const auto ok1 = VerifySizePrefixedHeaderBuffer(v1);
        REQUIRE(ok1 == true);
        Verifier v2(headerData, sizePrefix, 64U, 1000000U, false);
        const auto ok2 = VerifyHeaderBuffer(v2);
        REQUIRE(ok2 == true);
    }
}
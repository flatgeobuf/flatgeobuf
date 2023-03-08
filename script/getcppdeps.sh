#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v23.3.3.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 flatbuffers-23.3.3/include
curl -L https://github.com/mapbox/geojson-cpp/archive/v0.5.1.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 geojson-cpp-0.5.1/include
curl -L https://github.com/mapbox/geometry.hpp/archive/v1.1.0.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 geometry.hpp-1.1.0/include
curl -L https://github.com/mapbox/variant/archive/v1.1.4.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 variant-1.1.4/include
curl -L https://github.com/Tencent/rapidjson/archive/v1.1.0.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 rapidjson-1.1.0/include
curl -L https://github.com/catchorg/Catch2/releases/download/v2.13.10/catch.hpp -o src/cpp/test/catch.hpp

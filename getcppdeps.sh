#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v1.11.0.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 flatbuffers-1.11.0/include
curl -L https://github.com/mapbox/geojson-cpp/archive/v0.4.3.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 geojson-cpp-0.4.3/include
curl -L https://github.com/mapbox/geometry.hpp/archive/v1.0.0.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 geometry.hpp-1.0.0/include
curl -L https://github.com/mapbox/variant/archive/v1.1.4.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 variant-1.1.4/include
curl -L https://github.com/Tencent/rapidjson/archive/v1.1.0.tar.gz | \
    tar xz -C src/cpp/include --strip-components=2 rapidjson-1.1.0/include
curl -L https://github.com/catchorg/Catch2/releases/download/v2.7.2/catch.hpp -o src/cpp/test/catch.hpp
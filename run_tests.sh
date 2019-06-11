#!/bin/bash

echo -e '\033[1;33m--- Running TypeScript tests ---'
TS_NODE_PROJECT=tsconfig.test.json mocha -r ts-node/register -r esm src/**/*.spec.ts

echo -e '\033[1;33m--- Removing previous C++ tests if any ---'
rm -f ./testcpp

echo -e '\033[1;33m--- Compiling C++ tests ---'
clang++ -std=c++14 -Wall -Wfloat-conversion -Wmissing-declarations -g -o ./testcpp -Isrc/cpp/include src/cpp/packedrtree.cpp src/cpp/test/run_tests.cpp

echo -e '\033[1;33m--- Running C++ tests ---'
./testcpp -d yes

echo -e '\033[1;33m--- Running Java tests ---'
cd src/java && mvn test
#!/bin/bash

echo -e '\033[1;33m--- Running TypeScript tests ---'
mocha -r ts-node/register -r esm src/**/*.spec.ts

echo -e '\033[1;33m--- Compiling C++ tests ---'
g++ -o ./testcpp -Isrc/cpp src/cpp/test/run_tests.cpp

echo -e '\033[1;33m--- Running C++ tests ---'
./testcpp
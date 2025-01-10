#!/bin/bash
./flatc --csharp --gen-all --gen-object-api -o src/net/FlatGeobuf src/fbs/*.fbs
mv src/net/FlatGeobuf/FlatGeobuf/* src/net/FlatGeobuf
rmdir src/net/FlatGeobuf/FlatGeobuf

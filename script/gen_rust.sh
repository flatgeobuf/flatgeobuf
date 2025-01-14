#!/bin/bash
find src/fbs -type f -exec sed -i 's/namespace FlatGeobuf;/\/\/namespace FlatGeobuf;/g' {} +
./flatc --rust -o src/rust/src src/fbs/*.fbs
git checkout src/fbs

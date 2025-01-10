#!/bin/bash
./flatc --java --gen-all -o src/java/src/main/java/org/wololo/flatgeobuf/generated src/fbs/*.fbs
mv src/java/src/main/java/org/wololo/flatgeobuf/generated/FlatGeobuf/* src/java/src/main/java/org/wololo/flatgeobuf/generated
rmdir src/java/src/main/java/org/wololo/flatgeobuf/generated/FlatGeobuf
find src/java/src/main/java/org/wololo/flatgeobuf/generated -type f -exec sed -i 's/package FlatGeobuf;/package org.wololo.flatgeobuf.generated;/g' {} +
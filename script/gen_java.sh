#!/bin/bash
find src/fbs -type f -exec sed -i 's/namespace FlatGeobuf;/\/\/namespace FlatGeobuf;/g' {} +
./flatc --java --gen-all -o src/java/src/main/java/org/wololo/flatgeobuf/generated src/fbs/*.fbs
git checkout src/fbs
find src/java/src/main/java/org/wololo/flatgeobuf/generated -type f -exec sh -c 'content=$(cat "$1"); printf "%s\n%s" "package org.wololo.flatgeobuf.generated;" "$content" > "$1"' sh {} \;

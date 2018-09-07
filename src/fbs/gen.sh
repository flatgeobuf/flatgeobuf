#!/bin/bash
#./flatc --ts --gen-all -o src column.fbs
#./flatc --ts --no-ts-reexport -o src geometry.fbs
#./flatc --ts --no-ts-reexport -o src feature.fbs
./flatc --ts --gen-all -o src header.fbs
mv src/header_generated.ts src/flatgeobuf.ts
#!/bin/bash
#./flatc --ts --gen-all -o src column.fbs
#./flatc --ts --no-ts-reexport -o src geometry.fbs
#./flatc --ts --no-ts-reexport -o src feature.fbs
./flatc --cpp --scoped-enums -o src/cpp src/fbs/*.fbs

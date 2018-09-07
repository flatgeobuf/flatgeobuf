#!/bin/bash
./flatc --ts --gen-all -o ../fbs/header.fbs
mv ./header_generated.ts ./flatgeobuf.ts
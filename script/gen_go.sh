#!/bin/bash
./flatc --go --go-namespace flattypes -o src/go src/fbs/*.fbs

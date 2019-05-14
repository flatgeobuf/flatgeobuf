#!/bin/bash
flatc --ts --gen-all --no-ts-reexport --short-names --size-prefixed -o src/ts src/fbs/*.fbs

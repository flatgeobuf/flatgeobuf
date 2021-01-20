#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/master.tar.gz | tar xz --wildcards --strip-components=2 --directory=flatbuffers flatbuffers-master/ts/*.ts

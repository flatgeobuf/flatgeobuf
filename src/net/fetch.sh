#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v23.3.3.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-23.3.3/net/FlatBuffers/*.cs

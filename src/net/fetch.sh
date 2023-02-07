#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v23.1.21.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-23.1.21/net/FlatBuffers/*.cs

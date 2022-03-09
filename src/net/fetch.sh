#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v2.0.6.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-2.0.6/net/FlatBuffers/*.cs

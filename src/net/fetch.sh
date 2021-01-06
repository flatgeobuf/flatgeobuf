#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v1.12.0.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-1.12.0/net/FlatBuffers/*.cs

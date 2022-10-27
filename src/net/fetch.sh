#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v22.10.26.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-22.10.26/net/FlatBuffers/*.cs

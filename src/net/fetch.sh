#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v24.3.7.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-24.3.7/net/FlatBuffers/*.cs

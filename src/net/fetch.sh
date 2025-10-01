#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v25.9.23.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-24.12.23/net/FlatBuffers/*.cs

#!/bin/sh
curl -L https://github.com/google/flatbuffers/archive/v23.5.26.tar.gz | tar xz --wildcards --strip-components=2 --exclude="Properties" --directory=FlatGeobuf flatbuffers-23.5.26/net/FlatBuffers/*.cs

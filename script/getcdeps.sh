#!/bin/sh
curl -L https://github.com/dvidelabs/flatcc/archive/v0.6.0.tar.gz | \
    tar xz -C src/c/include --strip-components=2 flatcc-0.6.0/include

# flatgeobuf

[![CircleCI](https://circleci.com/gh/bjornharrtell/flatgeobuf.svg?style=svg)](https://circleci.com/gh/bjornharrtell/flatgeobuf)

A [flatbuffers](http://google.github.io/flatbuffers/) based performant binary encoding for geographic data.

Inspired by [geobuf](https://github.com/mapbox/geobuf) and [flatbush](https://github.com/mourner/flatbush). Deliberately does not support random writes for simplicity, to be able to use a static spatial index and to avoid fragementation issues.

DISCLAIMER: Unfinished work in progress.

## TODO

* Language support for JavaScript, TypeScript, C, C++, Java and C#
* Optional spatial index
* Optional attribute indexes (mabye, perhaps only unique id index?)
* Efficient I/O (streaming and random access)
* GDAL/OGR driver
* QGIS POC
* Complete test coverage

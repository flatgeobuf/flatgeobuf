# FlatGeobuf

[![CircleCI](https://img.shields.io/circleci/build/github/bjornharrtell/flatgeobuf.svg)](https://circleci.com/gh/bjornharrtell/flatgeobuf)
[![npm](https://img.shields.io/npm/v/flatgeobuf.svg)](https://www.npmjs.com/package/flatgeobuf)
[![Maven Central](https://img.shields.io/maven-central/v/org.wololo/flatgeobuf.svg)](https://search.maven.org/artifact/org.wololo/flatgeobuf)

A performant binary encoding for geographic data based on [flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features).

Inspired by [geobuf](https://github.com/mapbox/geobuf) and [flatbush](https://github.com/mourner/flatbush). Deliberately does not support random writes for simplicity and to be able to cluster the data on a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees) enabling fast bounding box spatial filtering.

Goals are to be suitable for large volumes of static data, significantly faster than legacy formats without size limitations for contents or metainformation and to be suitable for streaming/random access.

The site http://switchfromshapefile.org has more in depth information about the problems of legacy formats and provides some alternatives but acknowledges that the current alternatives has some drawbacks on their own, for example they are not suitable for streaming.

Live demonstration at https://observablehq.com/@bjornharrtell/streaming-flatgeobuf. (conceptual, not performance optimized)

DISCLAIMER: Implementation is now in a more or less finished state but specification remains non-final for now.

## Specification

![layout](doc/layout.svg "FlatGeobuf file layout")

* MB: Magic bytes (0x6667620066676200)
* H: Header (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/header.fbs))
* I+O (optional): Static packed Hilbert R-tree index (static size [custom buffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/cpp/packedrtree.h)) and feature offsets index (static size custom buffer, feature count * 8 bytes)
* DATA: Features (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/feature.fbs)s)

Any 64-bit type contained anywhere in the file (for example coordinate values) is aligned to 8 bytes to from the start of the file to allow for direct memory access.

## Performance

Preliminary performance tests has been done using road data from OSM for Denmark in SHP format from https://download.geofabrik.de/, containing 812547 LineString features with a set of attributes.

|                       | Shapefile | GeoPackage | FlatGeobuf | GeoJSON | GML |
|-----------------------|-----------|------------|------------|---------|-----|
| Read full dataset     | 1         | 0.9        | 0.5        | 15      | 7.7 |
| Read w/spatial filter | 1         | 0.15       | 0.12       | 100     | 60  |

The test was done using the GDAL fork (linked below) implementing FlatGeobuf as a driver and measurements of repeated reads using loops of `ogrinfo -qq` runs.

Note that for the test with spatial filter a small bounding box was chosen resulting in only 9 features. The reason for this is to test mainly the spatial index search performance for that case.

## Features

* Language support for JavaScript, TypeScript, C, C++, Java and C#
* Efficient I/O (streaming and random access)
* GDAL/OGR format (WIP @ https://github.com/bjornharrtell/gdal/tree/flatgeobuf)
* QGIS provider (WIP @ https://github.com/bjornharrtell/QGIS/tree/fgb)
* OpenLayers example (WIP @ https://github.com/bjornharrtell/ol3/tree/flatgeobuf)
* GeoServer WFS output format (WIP @ https://github.com/bjornharrtell/geoserver/tree/flatgeobuf-output)
* Complete test coverage

## TODO

* Finalize 1.0 spec
* Java index support
* C# support update
* C API
* Optimizations

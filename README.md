# ![layout](logo.svg) FlatGeobuf

[![CircleCI](https://img.shields.io/circleci/build/github/bjornharrtell/flatgeobuf.svg)](https://circleci.com/gh/bjornharrtell/flatgeobuf)
[![npm](https://img.shields.io/npm/v/flatgeobuf.svg)](https://www.npmjs.com/package/flatgeobuf)
[![Maven Central](https://img.shields.io/maven-central/v/org.wololo/flatgeobuf.svg)](https://search.maven.org/artifact/org.wololo/flatgeobuf)

A performant binary encoding for geographic data based on [flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features) including circular interpolations as defined by SQL-MM Part 3.

Inspired by [geobuf](https://github.com/mapbox/geobuf) and [flatbush](https://github.com/mourner/flatbush). Deliberately does not support random writes for simplicity and to be able to cluster the data on a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees) enabling fast bounding box spatial filtering. The spatial index is however optional to allow the format to be efficiently written as a stream.

Goals are to be suitable for large volumes of static data, significantly faster than legacy formats without size limitations for contents or metainformation and to be suitable for streaming/random access.

The site http://switchfromshapefile.org has more in depth information about the problems of legacy formats and provides some alternatives but acknowledges that the current alternatives has some drawbacks on their own, for example they are not suitable for streaming.

Live demonstration at https://observablehq.com/@bjornharrtell/streaming-flatgeobuf. (conceptual, not performance optimized)

## Specification

![layout](doc/layout.svg "FlatGeobuf file layout")

* MB: Magic bytes (0x6667620266676200)
* H: Header (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/header.fbs))
* I+O (optional): Static packed Hilbert R-tree index (static size [custom buffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/cpp/packedrtree.h)) and Feature offsets index (static size custom buffer, feature count * 8 bytes)
* DATA: Features (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/feature.fbs)s)

Any 64-bit flatbuffer value contained anywhere in the file (for example coordinate values) is aligned to 8 bytes to from the start of the file to allow for direct memory access.

## Performance

Preliminary performance tests has been done using road data from OSM for Denmark in SHP format from https://download.geofabrik.de/, containing 884594 LineString features with a set of attributes.

|                       | Shapefile | GeoPackage | FlatGeobuf | GeoJSON | GML |
|-----------------------|-----------|------------|------------|---------|-----|
| Read full dataset     | 1         | 0.94       | 0.47       | 14      | 7.9 |
| Read w/spatial filter | 1         | 0.14       | 0.13       | 102     | 53  |
| Write full dataset    | 1         | 0.78       | 0.40       | 3.1     | 3.2 |
| Write w/spatial index | 1         | 1.6        | 0.65       | -       | -   |
| Size                  | 1         | 0.72       | 0.76       | 1.2     | 2.1 |

The test was done using GDAL implementing FlatGeobuf as a driver and measurements for repeated reads using loops of `ogrinfo -qq -oo VERIFY_BUFFERS=NO` runs and measurements for repeated writes was done with `ogr2ogr` conversion from the original to a new file with `-lco SPATIAL_INDEX=NO` and `-lco SPATIAL_INDEX=YES` respectively.

Note that for the test with spatial filter a small bounding box was chosen resulting in only 16 features. The reason for this is to test mainly the spatial index search performance for that case.

## Features

* Language support for JavaScript, TypeScript, C, C++, Java and C#
* Efficient I/O (streaming and random access)
* [GDAL/OGR driver](https://gdal.org/drivers/vector/flatgeobuf.html)
* GeoServer WFS output format (https://docs.geoserver.org/latest/en/user/community/flatgeobuf/index.html)
* QGIS provider (WIP @ https://github.com/bjornharrtell/QGIS/tree/fgb)
* OpenLayers example (WIP @ https://github.com/bjornharrtell/ol3/tree/flatgeobuf)
* Complete test coverage

## TODO

* Java index support
* C# support update
* C langauge support
* Go langauge support
* Rust language support
* Further optimizations

## FAQ

### Why not use WKB geometry encoding?

It does not align on 8 bytes so it not always possible to consume it without copying first.

### Why not use Protobuf?

Performance reasons and to allow streaming/random access.

### Why am I not getting expected performance in GDAL?

Default behaviour is to assume untrusted data and verify buffer integrity for safety. If you have trusted data and want maximum performance make sure to set the open option VERIFY_BUFFERS to NO.

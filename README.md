# ![layout](logo.svg) FlatGeobuf

[![CircleCI](https://img.shields.io/circleci/build/github/bjornharrtell/flatgeobuf.svg)](https://circleci.com/gh/bjornharrtell/flatgeobuf)
[![npm](https://img.shields.io/npm/v/flatgeobuf.svg)](https://www.npmjs.com/package/flatgeobuf)
[![Maven Central](https://img.shields.io/maven-central/v/org.wololo/flatgeobuf.svg)](https://search.maven.org/artifact/org.wololo/flatgeobuf)
[![Nuget](https://img.shields.io/nuget/v/FlatGeobuf)](https://www.nuget.org/packages/FlatGeobuf/)
[![Crates.io](https://img.shields.io/crates/v/flatgeobuf.svg)](https://crates.io/crates/flatgeobuf)
[![Discord Chat](https://img.shields.io/discord/754359014917406730.svg)](https://discord.gg/GEHGxKx)
[![Twitter Follow](https://img.shields.io/twitter/follow/flatgeobuf.svg?style=social)](https://twitter.com/flatgeobuf)

A performant binary encoding for geographic data based on [flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features) including circular interpolations as defined by SQL-MM Part 3.

Inspired by [geobuf](https://github.com/mapbox/geobuf) and [flatbush](https://github.com/mourner/flatbush). Deliberately does not support random writes for simplicity and to be able to cluster the data on a [packed Hilbert R-Tree](https://en.wikipedia.org/wiki/Hilbert_R-tree#Packed_Hilbert_R-trees) enabling fast bounding box spatial filtering. The spatial index is optional to allow the format to be efficiently written as a stream and for use cases where spatial filtering is not needed.

Goals are to be suitable for large volumes of static data, significantly faster than legacy formats without size limitations for contents or metainformation and to be suitable for streaming/random access.

The site [switchfromshapefile.org](http://switchfromshapefile.org) has more in depth information about the problems of legacy formats and provides some alternatives but acknowledges that the current alternatives has some drawbacks on their own, for example they are not suitable for streaming.

## Examples

* [Observable notebook](https://observablehq.com/@bjornharrtell/streaming-flatgeobuf)
* [OpenLayers example](https://bjornharrtell.github.io/flatgeobuf/examples/openlayers)
* [Leaflet example](https://bjornharrtell.github.io/flatgeobuf/examples/leaflet)

## Specification

![layout](doc/layout.svg "FlatGeobuf file layout")

* MB: Magic bytes (0x6667620366676200)
* H: Header (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/header.fbs))
* I (optional): Static packed Hilbert R-tree index (static size [custom buffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/cpp/packedrtree.h))
* DATA: Features (variable size [flatbuffer](https://github.com/bjornharrtell/flatgeobuf/blob/master/src/fbs/feature.fbs)s)

Any 64-bit flatbuffer value contained anywhere in the file (for example coordinates) is aligned to 8 bytes to from the start of the file or feature to allow for direct memory access.

Encoding of any string value is assumed to be UTF-8.

## Performance

Preliminary performance tests has been done using road data from OSM for Denmark in SHP format from [download.geofabrik.de](https://download.geofabrik.de), containing 906602 LineString features with a set of attributes.

|                       | Shapefile | GeoPackage | FlatGeobuf | GeoJSON | GML |
|-----------------------|-----------|------------|------------|---------|-----|
| Read full dataset     | 1         | 1.02       | 0.46       | 15      | 8.9 |
| Read w/spatial filter | 1         | 0.94       | 0.71       | 705     | 399 |
| Write full dataset    | 1         | 0.77       | 0.39       | 3.9     | 3.2 |
| Write w/spatial index | 1         | 1.58       | 0.65       | -       | -   |
| Size                  | 1         | 0.72       | 0.77       | 1.2     | 2.1 |

The test was done using GDAL implementing FlatGeobuf as a driver and measurements for repeated reads using loops of `ogrinfo -qq -oo VERIFY_BUFFERS=NO` runs and measurements for repeated writes was done with `ogr2ogr` conversion from the original to a new file with `-lco SPATIAL_INDEX=NO` and `-lco SPATIAL_INDEX=YES` respectively.

Note that for the test with spatial filter a small bounding box was chosen resulting in only 1204 features. The reason for this is to primarily test the spatial index search performance.

As performance is highly data dependent I've also made similar tests on a larger dataset with Danish cadastral data consisting of 2511772 Polygons with extensive attribute data.

|                       | Shapefile | GeoPackage | FlatGeobuf | 
|-----------------------|-----------|------------|------------|
| Read full dataset     | 1         | 0.23       | 0.12       |
| Read w/spatial filter | 1         | 0.31       | 0.26       |
| Write full dataset    | 1         | 0.95       | 0.63       |
| Write w/spatial index | 1         | 1.07       | 0.70       |
| Size                  | 1         | 0.77       | 0.95       |

## Features

* Reference implementation for JavaScript, TypeScript, C++, C#, Java and Rust
* Efficient I/O (streaming and random access)
* [GDAL/OGR driver](https://gdal.org/drivers/vector/flatgeobuf.html)
* [GeoServer WFS output format](https://docs.geoserver.org/latest/en/user/community/flatgeobuf/index.html)

## Supported applications / libraries

* [Fiona](https://fiona.readthedocs.io/) (1.8.18 and forward)
* [GDAL](https://gdal.org) (3.1 and forward)
* [Geo Data Viewer (Visual Studio Code extension)](https://marketplace.visualstudio.com/items?itemName=RandomFractalsInc.geo-data-viewer) (2.3 and forward)
* [GeoServer](http://geoserver.org) (2.17 and forward)
* [QGIS](https://qgis.org) (3.16 and forward)

## Documentation

### TypeScript / JavaScript

* [API Docs](http://unpkg.com/flatgeobuf/dist/doc/modules/_flatgeobuf_.html)

#### Prebuilt bundles (intended for browser usage)

* [flatgeobuf.min.js](https://unpkg.com/flatgeobuf/dist/flatgeobuf.min.js) (contains the generic module)
* [flatgeobuf-geojson.min.js](https://unpkg.com/flatgeobuf/dist/flatgeobuf-geojson.min.js) (contains the geojson module)
* [flatgeobuf-ol.min.js](https://unpkg.com/flatgeobuf/dist/flatgeobuf-ol.min.js) (contains the ol module)

### Node usage

See [this](examples/node) example for a minimal how to depend on and use the flatgeobuf npm package.

## TODO

* Java index support
* C language support
* Go language support

## FAQ

### Why not use WKB geometry encoding?

It does not align on 8 bytes so it not always possible to consume it without copying first.

### Why not use Protobuf?

Performance reasons and to allow streaming/random access.

### Why am I not getting expected performance in GDAL?

Default behaviour is to assume untrusted data and verify buffer integrity for safety. If you have trusted data and want maximum performance make sure to set the open option VERIFY_BUFFERS to NO.

### What about vector tiles?

FlatGeobuf does not aim to compete with vector tiles. Vector tiles are great for rendering but they are relatively expensive to create and is a lossy format, where as FlatGeobuf is lossless and very fast to write especially if a spatial index is not needed.

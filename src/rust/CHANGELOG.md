# Changelog

## [4.4.0] - 2024-09-26

- [rust] Upgrade to geozero 0.14

## [4.3.0] - 2024-08-16

- [all] Upgrade to flatbuffers 24.3.25 (#380)
- Update flatbuffers, reqwest, yocalhost (#378)
- Fix too-small request sizing after making a large request. (#376)

## [4.2.1] - 2024-06-17

- Fix WASM build for Rust (#366)

## [4.2.0] - 2024-05-28

- Fix "UnexpectedEof" error when bbox results includes first item.
- Upgrade to geozero 0.13.0

## [4.1.0] - 2024-02-24

- Potentially reduce requests for feature data by correctly including distance
  to the next feature in the index traversal in all cases.
- Fix HTTP 416 on webservers that don't support Range requests that extend
  beyond the end of the file. This was only applicable to bbox requests.
- Upgrade to geozero 0.12.0
- Make HttpFgbReader generic

## [4.0.0] - 2023-10-14

- Breaking: `select_all` and `select_bbox` now return a FeatureIter instead of a
  modified Self type.
- Ensure reading from tls is supported.
- Breaking: Added flatgeobuf::Error with more specific errors and use it where possible
  rather than geozero::GeozeroError.
- Added feature batching for HTTP client to optimize remote reading.

## [3.27.0] - 2023-08-28

- Fix: Columns of type `short` are now correctly 16 bit, previously they were
  considered to be 8 bits, resulting in unexpected overflow.
  - Note that this fix could cause newly appearing breaks in behavior if you
    have a legacy FGB file (written with version `<= 3.26.0`) which contains a
    `short` column, because all subsequent property columns would be
    incorrectly offset.
- Fix `size` inconsistency with `linestring_begin` (#287)
- Breaking: Remove lifetimes from FgbReader/FgbWriter
- FgbReader/FgbWriter as well as borrow, can now own their inner reader/writer
- Breaking: Replace pub dim field of FeatureWriter with constructor
- Bump deps & 2021 edition, reexports, clippy (#282)
- Upgrade to geozero 0.11

## [3.26.1] - 2023-07-19

- Fix inconsistent result ordering (#279)

## [3.26.0] - 2023-07-08

- Upgrade to geozero 0.10

## [3.25.0] - 2023-03-08

- Upgrade to Flatbuffers 23.1.21

## [3.24.0] - 2022-11-04

- Upgrade to Flatbuffers 22.10.26

## [0.8.0] - 2022-05-04

- Breaking: New create methods for FgbWriter, with or without options
- Optional conversion from single to multi geometry types
- Support reading files with undefined feature count
  - Breaking: features_count returns None if undefined
- Support for file reading without seek
- Make reader state types public
- Writer: Fix bounding boxes in index
- Writer: Reduced file size
- Update to geozero 0.9

## [0.7.0] - 2022-03-14

- Add explicit reader/writer state to avoid wrong API use
  - Breaking: select_all/select_bbox now return the reader struct
- Optional reading without FlatBuffers verification
- Handle empty columns in header in rust reader
- Support GeometryCollection in writer
- Update to geozero 0.8.0

## [0.6.2] - 2021-11-19

- Write support for basic geometry types
- Fix reading FGB without index or properties

## [0.6.1] - 2021-10-02

- Make all impl. lenient on magic bytes patch level (#146)

## [0.6.0] - 2021-09-01

- Update to Rust Flatbuffers 2.0 (#105)
- Verify Flatbuffers when reading
- Indicate correct license
- Use seek_bufread::BufReader in benches (#111)
- Drop driver trait impl
- Impl GeozeroDatasource for FgbReader
- Update to geozero 0.7
- Make http an optional feature

## [0.5.0] - 2021-02-26

- Disable default features of reqwest

## [0.4.1] - 2021-01-26

- Impl FeatureAccess traits
- Add property access functions
- Impl FallibleStreamingIterator

## [0.4.0] - 2022-12-24

- Rename HttpClient to HttpRangeClient
- Make smaller index requests, merging where possible
- Prefetch some index layers
- Fix crashing bug in HttpClient
- Log network usage (adds log crate)
- Avoid FlatBuffers panic caused by malicious header data (#86)
- Fix memory exhaustion with malicious header size (#85)
- Add fuzz target for feature reading
- Add fuzz target for the Rust crate. (#84)
- Additional metadata fields (#75)

## [0.3.4] - 2020-08-12

- Fix WASM build

## [0.3.3] - 2020-08-11

- Add support for triangle/polyhedralsurface/tin
- Add support for curve types
- Add support for GeometryCollection type

## [0.3.2] - 2020-05-11

- Rust API and index improvements (#54)

## [0.3.1] - 2020-04-05

- Rust FlatGeobuf reading via HTTP (#49)
- Add Rust docs URL and update README files (#48)

## [0.3.0] - 2020-03-20

- Rust implementation (#47)

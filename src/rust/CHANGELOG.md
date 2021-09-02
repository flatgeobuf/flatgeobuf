# Changelog

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
- Add Rust docs URL and update READMEs (#48)

## [0.3.0] - 2020-03-20

- Rust implementation (#47)

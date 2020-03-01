# FlatGeobuf for Rust

Rust implementation of [FlatGeobuf](https://bjornharrtell.github.io/flatgeobuf/).

## Limitations

- No read/write support for packed R-tree index
- Missing FlatBuffers features like simple mutation and buffer verifier
  ([Platform / Language / Feature support](http://google.github.io/flatbuffers/flatbuffers_support.html))

## Documentation

    cargo doc --open

## Usage

See [tests](tests/)

## Run tests

    cargo test

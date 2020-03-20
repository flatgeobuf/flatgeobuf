# FlatGeobuf for Rust

Rust implementation of [FlatGeobuf](https://bjornharrtell.github.io/flatgeobuf/).

FlatGeobuf is a performant binary encoding for geographic data based on
[flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection
of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features) including
circular interpolations as defined by SQL-MM Part 3.

## Usage

```rust
use flatgeobuf::*;

let mut reader = Reader::new(File::open("countries.fgb")?);
let header = reader.read_header()?;
let columns_meta = columns_meta(&header);

reader.select_bbox(8.8, 47.2, 9.5, 55.3)?;
while let Ok(feature) = reader.next() {
    let props = read_all_properties(&feature, &columns_meta);
    println!("{}", props["name"]);
}
```

## Documentation

    cargo doc --open

## Usage

See [tests](tests/)

## Run tests

    cargo test

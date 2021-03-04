# FlatGeobuf for Rust

Rust implementation of [FlatGeobuf](https://flatgeobuf.org/).

FlatGeobuf is a performant binary encoding for geographic data based on
[flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection
of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features) including
circular interpolations as defined by SQL-MM Part 3.

## Usage

```rust
use flatgeobuf::*;

let mut filein = BufReader::new(File::open("countries.fgb")?);
let mut fgb = FgbReader::open(&mut filein)?;
fgb.select_all()?;
while let Some(feature) = fgb.next()? {
    println!("{}", feature.property::<String>("name").unwrap());
    println!("{}", feature.to_json()?);
}
```

With async HTTP client:
```rust
use flatgeobuf::*;

let mut fgb = HttpFgbReader::open("https://flatgeobuf.org/test/data/countries.fgb").await?;
fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
while let Some(feature) = fgb.next().await? {
    let props = feature.properties()?;
    println!("{}", props["name"]);
    println!("{}", feature.to_wkt()?);
}
```

See [documentation](https://docs.rs/flatgeobuf/) and [tests](https://github.com/flatgeobuf/flatgeobuf/tree/master/src/rust/tests) for more examples.

## Run tests and benchmarks

    cargo test

    cargo criterion

## Run fuzzer

    cargo install cargo-fuzz

    cargo +nightly fuzz run read

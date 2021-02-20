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
fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
while let Some(feature) = fgb.next()? {
    let props = feature.properties()?;
    println!("{}", props["name"]);
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
}
```

See [documentation](https://docs.rs/flatgeobuf/) and [tests](tests/) for more examples.

## Run tests and benchmarks

    cargo test

    cargo bench

## Run fuzzer

    cargo install cargo-fuzz

    cargo +nightly fuzz run read

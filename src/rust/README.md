# FlatGeobuf for Rust

Rust implementation of [FlatGeobuf](https://bjornharrtell.github.io/flatgeobuf/).

FlatGeobuf is a performant binary encoding for geographic data based on
[flatbuffers](http://google.github.io/flatbuffers/) that can hold a collection
of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features) including
circular interpolations as defined by SQL-MM Part 3.

## Usage

```rust
use flatgeobuf::*;

let mut file = BufReader::new(File::open("countries.fgb")?);
let hreader = HeaderReader::read(&mut file)?;
let header = hreader.header();

let mut freader = FeatureReader::select_bbox(&mut file, &header, 8.8, 47.2, 9.5, 55.3)?;
while let Ok(feature) = freader.next(&mut file) {
    let props = feature.properties_map(&header);
    println!("{}", props["name"]);
}
```

With async HTTP client:
```rust
use flatgeobuf::*;

let mut client = BufferedHttpClient::new("https://pkg.sourcepole.ch/countries.fgb");
let hreader = HttpHeaderReader::read(&mut client).await.unwrap();
let header = hreader.header();

let mut freader = HttpFeatureReader::select_all(&header, hreader.header_len()).await?;
while let Ok(feature) = freader.next(&mut client).await {
    let props = feature.properties_map(&header);
    println!("{}", props["name"]);
}
```

See [documentation](https://docs.rs/flatgeobuf/) and [tests](tests/) for more examples.

## Run tests and benchmarks

    cargo test

    cargo bench

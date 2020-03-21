//! [FlatGeobuf](https://bjornharrtell.github.io/flatgeobuf/) is a performant binary encoding
//! for geographic data based on [flatbuffers](http://google.github.io/flatbuffers/) that
//! can hold a collection of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features)
//! including circular interpolations as defined by SQL-MM Part 3.
//!
//!
//! ## Installation
//!
//! ```ini
//! [dependencies]
//! flatgeobuf = "0.3"
//! ```
//!
//! ## Reading a FlatGeobuf file
//!
//! ```rust
//! use flatgeobuf::*;
//! # use std::fs::File;
//! # use std::io::BufReader;
//!
//! # fn read_fbg() -> std::result::Result<(), std::io::Error> {
//! let mut file = BufReader::new(File::open("countries.fgb")?);
//! let hreader = HeaderReader::read(&mut file)?;
//! let header = hreader.header();
//!
//! let mut freader = FeatureReader::select_bbox(&mut file, &header, 8.8, 47.2, 9.5, 55.3)?;
//! while let Ok(feature) = freader.next(&mut file) {
//!     let props = feature.properties_map(&header);
//!     println!("{}", props["name"]);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Zero-copy feature access
//!
//! ```rust
//! # use flatgeobuf::*;
//! # use std::fs::File;
//! # use std::io::BufReader;
//! # let fgb = File::open("../../test/data/countries.fgb").unwrap();
//! # let mut reader = BufReader::new(fgb);
//! # let hreader = HeaderReader::read(&mut reader).unwrap();
//! # let header = hreader.header();
//! # let mut freader = FeatureReader::select_all(&mut reader, &header).unwrap();
//! # let feature = freader.next(&mut reader).unwrap();
//! let _ = feature.iter_properties(&header, |i, n, v| {
//!     println!("columnidx: {} name: {} value: {:?}", i, n, v);
//!     false // don't abort
//! });
//! ```
//!
//! ## Zero-copy geometry reader
//!
//! Geometries can be accessed by implementing the `GeomReader` trait.
//!
//! ```rust
//! # use flatgeobuf::*;
//! # use std::fs::File;
//! # use std::io::BufReader;
//! struct CoordPrinter;
//!
//! impl GeomReader for CoordPrinter {
//!     fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
//!         println!("({} {})", x, y);
//!     }
//! }
//!
//! # let fgb = File::open("../../test/data/countries.fgb").unwrap();
//! # let mut reader = BufReader::new(fgb);
//! # let hreader = HeaderReader::read(&mut reader).unwrap();
//! # let header = hreader.header();
//! # let mut freader = FeatureReader::select_all(&mut reader, &header).unwrap();
//! # let feature = freader.next(&mut reader).unwrap();
//! let mut coord_printer = CoordPrinter {};
//! let geometry = feature.geometry().unwrap();
//! geometry.parse(&mut coord_printer, header.geometry_type());
//! ```

#[allow(dead_code, unused_imports, non_snake_case)]
mod feature_generated;
mod geojson;
#[allow(dead_code, unused_imports, non_snake_case)]
mod header_generated;
mod packed_r_tree;
mod reader;
mod svg;

pub use feature_generated::flat_geobuf::*;
pub use geojson::*;
pub use header_generated::flat_geobuf::*;
pub use packed_r_tree::*;
pub use reader::*;
pub use svg::*;

pub const VERSION: u8 = 3;
pub const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

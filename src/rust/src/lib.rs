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
//! flatgeobuf = "3.0"
//! ```
//!
//! ## Reading a FlatGeobuf file
//!
//! ```rust
//! use flatgeobuf::*;
//! # use std::fs::File;
//!
//! # let fgb = "../../test/data/countries.fgb";
//! let mut reader = Reader::new(File::open(fgb).unwrap());
//! let header = reader.read_header().unwrap();
//! let columns_meta = columns_meta(&header);
//!
//! reader.select_bbox(8.8, 47.2, 9.5, 55.3).unwrap();
//! while let Ok(feature) = reader.next() {
//!     let props = read_all_properties(&feature, &columns_meta);
//!     println!("{}", props["name"]);
//! }
//! ```

#[allow(dead_code, unused_imports, non_snake_case)]
mod feature_generated;
#[allow(dead_code, unused_imports, non_snake_case)]
mod header_generated;
mod packed_r_tree;
mod reader;

pub use feature_generated::flat_geobuf::*;
pub use header_generated::flat_geobuf::*;
pub use packed_r_tree::PackedRTree;
pub use reader::*;

pub const VERSION: u8 = 3;
pub const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

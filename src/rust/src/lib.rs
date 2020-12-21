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
//! # fn read_fbg() -> geozero::error::Result<()> {
//! let mut filein = BufReader::new(File::open("countries.fgb")?);
//! let mut fgb = FgbReader::open(&mut filein)?;
//! fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
//! while let Some(feature) = fgb.next()? {
//!     let props = feature.properties()?;
//!     println!("{}", props["name"]);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Zero-copy geometry reader
//!
//! Geometries can be accessed by implementing the `GeomProcessor` trait.
//!
//! ```rust
//! use geozero::{GeomProcessor, error::Result};
//! # use flatgeobuf::*;
//! # use std::fs::File;
//! # use std::io::BufReader;
//!
//! struct CoordPrinter;
//!
//! impl GeomProcessor for CoordPrinter {
//!     fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
//!         println!("({} {})", x, y);
//!         Ok(())
//!     }
//! }
//!
//! # fn read_fbg() -> geozero::error::Result<()> {
//! # let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
//! # let mut fgb = FgbReader::open(&mut filein)?;
//! # let geometry_type = fgb.header().geometry_type();
//! # fgb.select_all()?;
//! # let feature = fgb.next()?.unwrap();
//! let mut coord_printer = CoordPrinter {};
//! let geometry = feature.geometry().unwrap();
//! geometry.process(&mut coord_printer, geometry_type)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Zero-copy feature access
//!
//! Properties can be accessed by implementing the `PropertyProcessor` trait.
//!
//! ```rust
//! use geozero::{PropertyProcessor, ColumnValue, error::Result};
//! # use flatgeobuf::*;
//! # use std::fs::File;
//! # use std::io::BufReader;
//!
//! struct PropertyPrinter;
//!
//! impl PropertyProcessor for PropertyPrinter {
//!     fn property(&mut self, i: usize, n: &str, v: &ColumnValue) -> Result<bool> {
//!         println!("columnidx: {} name: {} value: {:?}", i, n, v);
//!         Ok(false) // don't abort
//!     }
//! }
//!
//! # fn read_fbg() -> geozero::error::Result<()> {
//! # let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
//! # let mut fgb = FgbReader::open(&mut filein)?;
//! # fgb.select_all()?;
//! # let feature = fgb.next()?.unwrap();
//! let mut prop_printer = PropertyPrinter {};
//! let _ = feature.process_properties(&mut prop_printer)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Reading FlatGeobuf via HTTP
//!
//! ```rust
//! use flatgeobuf::*;
//!
//! # async fn read_fbg() -> geozero::error::Result<()> {
//! let mut fgb = HttpFgbReader::open("https://pkg.sourcepole.ch/countries.fgb").await?;
//! fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
//! while let Some(feature) = fgb.next().await? {
//!     let props = feature.properties()?;
//!     println!("{}", props["name"]);
//! }
//! # Ok(())
//! # }
//! ```
//!

#[macro_use]
extern crate log;

#[cfg(not(target_arch = "wasm32"))]
mod driver;
#[allow(dead_code, unused_imports, non_snake_case)]
mod feature_generated;
mod file_reader;
mod geometry_reader;
#[allow(dead_code, unused_imports, non_snake_case)]
mod header_generated;
mod http_client;
mod http_reader;
mod packed_r_tree;
mod properties_reader;

#[cfg(not(target_arch = "wasm32"))]
pub use driver::*;
pub use feature_generated::flat_geobuf::*;
pub use file_reader::*;
pub use geometry_reader::*;
pub use header_generated::flat_geobuf::*;
pub use http_client::*;
pub use http_reader::*;
pub use packed_r_tree::*;
pub use properties_reader::*;

pub const VERSION: u8 = 3;
pub const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

pub const HEADER_MAX_BUFFER_SIZE: usize = 1048576 * 10;

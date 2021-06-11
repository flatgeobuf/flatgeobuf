//! [FlatGeobuf](https://flatgeobuf.org/) is a performant binary encoding
//! for geographic data based on [flatbuffers](http://google.github.io/flatbuffers/) that
//! can hold a collection of [Simple Features](https://en.wikipedia.org/wiki/Simple_Features)
//! including circular interpolations as defined by SQL-MM Part 3.
//!
//!
//! ## Reading a FlatGeobuf file
//!
//! ```rust
//! use flatgeobuf::*;
//! use geozero::ToJson;
//! # use std::fs::File;
//! # use std::io::BufReader;
//!
//! # fn read_fbg() -> geozero::error::Result<()> {
//! let mut filein = BufReader::new(File::open("countries.fgb")?);
//! let mut fgb = FgbReader::open(&mut filein)?;
//! fgb.select_all()?;
//! while let Some(feature) = fgb.next()? {
//!     println!("{}", feature.property::<String>("name").unwrap());
//!     println!("{}", feature.to_json()?);
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
//! # fgb.select_all()?;
//! # let feature = fgb.next()?.unwrap();
//! let mut coord_printer = CoordPrinter {};
//! feature.process_geom(&mut coord_printer)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Reading FlatGeobuf via HTTP
//!
//! ```rust
//! use flatgeobuf::*;
//! use geozero::ToWkt;
//!
//! # #[cfg(feature = "http")]
//! # async fn read_fbg() -> geozero::error::Result<()> {
//! let mut fgb = HttpFgbReader::open("https://flatgeobuf.org/test/data/countries.fgb").await?;
//! fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
//! while let Some(feature) = fgb.next().await? {
//!     let props = feature.properties()?;
//!     println!("{}", props["name"]);
//!     println!("{}", feature.to_wkt()?);
//! }
//! # Ok(())
//! # }
//! ```
//!

#[cfg(feature = "http")]
#[macro_use]
extern crate log;

#[allow(unused_imports, non_snake_case)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod feature_generated;
mod file_reader;
mod geometry_reader;
#[allow(unused_imports, non_snake_case)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod header_generated;
#[cfg(feature = "http")]
mod http_client;
#[cfg(feature = "http")]
mod http_reader;
mod packed_r_tree;
mod properties_reader;

pub use feature_generated::*;
pub use file_reader::*;
pub use geometry_reader::*;
pub use header_generated::*;
#[cfg(feature = "http")]
pub use http_client::*;
#[cfg(feature = "http")]
pub use http_reader::*;
pub use packed_r_tree::*;
pub use properties_reader::*;

// Re-export used traits
pub use fallible_streaming_iterator::FallibleStreamingIterator;
pub use geozero::{FeatureAccess, FeatureProperties, GeozeroGeometry};

pub const VERSION: u8 = 3;
const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

const HEADER_MAX_BUFFER_SIZE: usize = 1048576 * 10;

fn check_magic_bytes(magic_bytes: &[u8]) -> bool {
    magic_bytes[0..3] == MAGIC_BYTES[0..3]
        && magic_bytes[4..8] == MAGIC_BYTES[4..8]
        && magic_bytes[3] <= VERSION
}

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
//! let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
//! while let Some(feature) = fgb.next()? {
//!     println!("{}", feature.property::<String>("name").unwrap());
//!     println!("{}", feature.to_json()?);
//! }
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
//! let mut fgb = HttpFgbReader::open("https://flatgeobuf.org/test/data/countries.fgb")
//!     .await?
//!     .select_bbox(8.8, 47.2, 9.5, 55.3)
//!     .await?;
//! while let Some(feature) = fgb.next().await? {
//!     let props = feature.properties()?;
//!     println!("{}", props["name"]);
//!     println!("{}", feature.to_wkt()?);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Writing a FlatGeobuf file
//!
//! ```rust
//! use flatgeobuf::*;
//! use geozero::geojson::GeoJsonReader;
//! use geozero::GeozeroDatasource;
//! # use std::fs::File;
//! # use std::io::{BufReader, BufWriter};
//!
//! # fn json_to_fgb() -> geozero::error::Result<()> {
//! let mut fgb = FgbWriter::create("countries", GeometryType::MultiPolygon, |_, _| {})?;
//! let mut fin = BufReader::new(File::open("countries.geojson")?);
//! let mut reader = GeoJsonReader(&mut fin);
//! reader.process(&mut fgb)?;
//! let mut fout = BufWriter::new(File::create("countries.fgb")?);
//! fgb.write(&mut fout)?;
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
mod feature_writer;
mod file_reader;
mod file_writer;
mod geometry_reader;
#[allow(unused_imports, non_snake_case)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod header_generated;
#[cfg(feature = "http")]
mod http_reader;
pub mod packed_r_tree;
mod properties_reader;

pub use feature_generated::*;
pub use file_reader::*;
pub use file_writer::*;
pub use geometry_reader::*;
pub use header_generated::*;
#[cfg(feature = "http")]
pub use http_reader::*;
pub use properties_reader::*;

// Reader states for ensuring correct read API usage at compile-time
#[doc(hidden)]
pub mod reader_state {
    pub struct Initial;
    pub struct Open;
    pub struct FeaturesSelected;
    pub struct FeaturesSelectedSeek;
}

// Re-export used traits
pub use fallible_streaming_iterator::FallibleStreamingIterator;
pub use geozero::{FeatureAccess, FeatureProperties, GeozeroGeometry};

pub const VERSION: u8 = 3;
pub(crate) const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

const HEADER_MAX_BUFFER_SIZE: usize = 1048576 * 10;

fn check_magic_bytes(magic_bytes: &[u8]) -> bool {
    magic_bytes[0..3] == MAGIC_BYTES[0..3]
        && magic_bytes[4..7] == MAGIC_BYTES[4..7]
        && magic_bytes[3] <= VERSION
}

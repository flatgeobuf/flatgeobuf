#[allow(dead_code, unused_imports, non_snake_case)]
mod feature_generated;
#[allow(dead_code, unused_imports, non_snake_case)]
mod header_generated;
mod reader;

pub use feature_generated::flat_geobuf::*;
pub use header_generated::flat_geobuf::*;
pub use reader::*;

pub const VERSION: u8 = 3;
pub const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0];

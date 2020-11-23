#![no_main]

use flatgeobuf::*;
use libfuzzer_sys::fuzz_target;
use std::io;

fuzz_target!(|data: &[u8]| {
    let mut buf_reader = io::BufReader::new(io::Cursor::new(data));
    let mut fgb = match FgbReader::open(&mut buf_reader) {
        Ok(n) => n,
        Err(_) => return,
    };
    let _ = fgb.header();
    while let Ok(Some(feature)) = fgb.next() {
    }
});

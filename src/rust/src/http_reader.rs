use crate::header_generated::flat_geobuf::*;
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::io::{Error, ErrorKind};
use std::str;

pub struct HttpClient<'a> {
    url: &'a str,
}

impl<'a> HttpClient<'a> {
    pub fn new(url: &'a str) -> Self {
        HttpClient { url }
    }
    pub fn get(&self, begin: usize, length: usize) -> Result<Bytes, std::io::Error> {
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(self.url)
            .header("Range", format!("bytes={}-{}", begin, begin + length - 1))
            .send()
            .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Response with status {}", response.status()),
            ));
        }
        response
            .bytes()
            .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))
    }
}

pub struct HttpHeaderReader {
    bytes: Bytes,
}

impl HttpHeaderReader {
    pub fn read(client: &HttpClient) -> Result<Self, std::io::Error> {
        let bytes = client.get(0, 12)?;
        assert_eq!(bytes.len(), 12);
        let mut data = HttpHeaderReader { bytes };
        if data.bytes[0..8] != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }

        let header_size = LittleEndian::read_u32(&data.bytes[8..12]) as usize;
        data.bytes = client.get(12, header_size)?;

        assert_eq!(data.bytes.len(), header_size);
        Ok(data)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.bytes[..])
    }
}

use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use std::io::{Error, ErrorKind};
use std::str;

pub struct HttpClient<'a> {
    client: reqwest::Client,
    url: &'a str,
}

impl<'a> HttpClient<'a> {
    pub fn new(url: &'a str) -> Self {
        HttpClient {
            client: reqwest::Client::new(),
            url,
        }
    }
    pub async fn get(&self, begin: usize, length: usize) -> Result<Bytes, std::io::Error> {
        let response = self
            .client
            .get(self.url)
            .header("Range", format!("bytes={}-{}", begin, begin + length - 1))
            .send()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))?;
        if !response.status().is_success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Response with status {}", response.status()),
            ));
        }
        response
            .bytes()
            .await
            .map_err(|e| Error::new(ErrorKind::Other, format!("{}", e)))
    }
}

pub struct HttpHeaderReader {
    bytes: Bytes,
}

impl HttpHeaderReader {
    pub async fn read(client: &HttpClient<'_>) -> Result<Self, std::io::Error> {
        let bytes = client.get(0, 12).await?;
        assert_eq!(bytes.len(), 12);
        let mut data = HttpHeaderReader { bytes };
        if data.bytes[0..8] != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }

        let header_size = LittleEndian::read_u32(&data.bytes[8..12]) as usize;
        data.bytes = client.get(12, header_size).await?;

        assert_eq!(data.bytes.len(), header_size);
        Ok(data)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.bytes[..])
    }
    pub fn header_len(&self) -> usize {
        12 + self.bytes.len()
    }
}

/// FlatGeobuf feature reader
pub struct HttpFeatureReader {
    feature_base: usize,
    pos: usize,
    feature_buf: Bytes,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Current position in item_filter
    filter_idx: usize,
}

impl HttpFeatureReader {
    /// Skip R-Tree index
    pub async fn select_all(
        header: &Header<'_>,
        header_len: usize,
    ) -> std::result::Result<Self, std::io::Error> {
        // Skip index
        let index_size =
            PackedRTree::index_size(header.features_count() as usize, header.index_node_size());
        let feature_base = header_len + index_size;
        let data = HttpFeatureReader {
            feature_base,
            pos: feature_base,
            feature_buf: Bytes::new(),
            item_filter: None,
            filter_idx: 0,
        };
        Ok(data)
    }
    /// Number of selected features
    pub fn filter_count(&self) -> Option<usize> {
        self.item_filter.as_ref().map(|f| f.len())
    }
    /// Read next feature
    pub async fn next(
        &mut self,
        client: &HttpClient<'_>,
    ) -> std::result::Result<Feature<'_>, std::io::Error> {
        if let Some(filter) = &self.item_filter {
            if self.filter_idx >= filter.len() {
                return Err(Error::new(ErrorKind::Other, "No more features"));
            }
            let item = &filter[self.filter_idx];
            self.pos = self.feature_base + item.offset;
            self.filter_idx += 1;
        }
        let bytes = client.get(self.pos, 4).await?;
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(&bytes) as usize;
        self.feature_buf = client.get(self.pos, feature_size).await?;
        self.pos += feature_size;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
    }
    /// Return current feature
    pub fn cur_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
}

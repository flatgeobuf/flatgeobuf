use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use std::cmp::max;
use std::io::{Error, ErrorKind};
use std::str;

struct HttpClient<'a> {
    client: reqwest::Client,
    url: &'a str,
}

impl<'a> HttpClient<'a> {
    fn new(url: &'a str) -> Self {
        HttpClient {
            client: reqwest::Client::new(),
            url,
        }
    }
    async fn get(&self, begin: usize, length: usize) -> Result<Bytes, std::io::Error> {
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

pub struct BufferedHttpClient<'a> {
    http_client: HttpClient<'a>,
    buf: BytesMut,
    /// Lower index of buffer relative to input stream
    head: usize,
}

impl<'a> BufferedHttpClient<'a> {
    pub fn new(url: &'a str) -> Self {
        BufferedHttpClient {
            http_client: HttpClient::new(url),
            buf: BytesMut::new(),
            head: 0,
        }
    }
    pub async fn get(
        &mut self,
        begin: usize,
        length: usize,
        min_req_size: usize,
    ) -> Result<&[u8], std::io::Error> {
        let tail = self.head + self.buf.len();
        if begin + length > tail || begin < self.head {
            // Remove bytes before new begin
            if begin > self.head && begin < tail {
                let _ = self.buf.split_to(begin - self.head);
                self.head = begin;
            } else if begin >= tail || begin < self.head {
                self.buf.clear();
                self.head = begin;
            }

            // Read additional bytes
            let range_begin = max(begin, tail);
            let range_length = max(begin + length - range_begin, min_req_size);
            let bytes = self.http_client.get(range_begin, range_length).await?;
            self.buf.put(bytes);
        }
        let lower = begin - self.head;
        let upper = begin + length - self.head;
        Ok(&self.buf[lower..upper])
    }
}

pub struct HttpHeaderReader {
    header_buf: Vec<u8>,
}

impl HttpHeaderReader {
    pub async fn read(client: &mut BufferedHttpClient<'_>) -> Result<Self, std::io::Error> {
        let min_req_size = 512;
        let bytes = client.get(0, 8, min_req_size).await?;
        if bytes != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }
        let bytes = client.get(8, 12, min_req_size).await?;
        let header_size = LittleEndian::read_u32(bytes) as usize;
        let bytes = client.get(12, header_size, min_req_size).await?;
        let data = HttpHeaderReader {
            header_buf: bytes.to_vec(),
        };
        Ok(data)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.header_buf[..])
    }
    pub fn header_len(&self) -> usize {
        12 + self.header_buf.len()
    }
}

/// FlatGeobuf feature reader
pub struct HttpFeatureReader {
    feature_base: usize,
    pos: usize,
    feature_buf: Vec<u8>,
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
        let index_size =
            PackedRTree::index_size(header.features_count() as usize, header.index_node_size());
        // Skip index
        let feature_base = header_len + index_size;
        let reader = HttpFeatureReader {
            feature_base,
            pos: feature_base,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        };
        Ok(reader)
    }
    /// Read R-Tree index and build filter for features within bbox
    pub async fn select_bbox(
        mut client: &mut BufferedHttpClient<'_>,
        header: &Header<'_>,
        header_len: usize,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> std::result::Result<Self, std::io::Error> {
        let tree = PackedRTree::from_http(
            &mut client,
            header_len,
            header.features_count() as usize,
            PackedRTree::DEFAULT_NODE_SIZE,
        )
        .await;
        let feature_base = header_len + tree.size();
        let mut list = tree.search(min_x, min_y, max_x, max_y);
        list.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        let reader = HttpFeatureReader {
            feature_base,
            pos: feature_base,
            feature_buf: Vec::new(),
            item_filter: Some(list),
            filter_idx: 0,
        };
        Ok(reader)
    }
    /// Number of selected features
    pub fn filter_count(&self) -> Option<usize> {
        self.item_filter.as_ref().map(|f| f.len())
    }
    /// Read next feature
    pub async fn next(
        &mut self,
        client: &mut BufferedHttpClient<'_>,
    ) -> std::result::Result<Feature<'_>, std::io::Error> {
        let min_req_size = 1_048_576; // 1MB
        if let Some(filter) = &self.item_filter {
            if self.filter_idx >= filter.len() {
                return Err(Error::new(ErrorKind::Other, "No more features"));
            }
            let item = &filter[self.filter_idx];
            self.pos = self.feature_base + item.offset;
            self.filter_idx += 1;
        }
        let bytes = client.get(self.pos, 4, min_req_size).await?;
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(bytes) as usize;
        let bytes = client.get(self.pos, feature_size, min_req_size).await?;
        self.feature_buf = bytes.to_vec(); // Not zero-copy
        self.pos += feature_size;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
    }
    /// Return current feature
    pub fn cur_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
}

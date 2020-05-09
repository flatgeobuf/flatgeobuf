use crate::header_generated::flat_geobuf::*;
use crate::http_client::BufferedHttpClient;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use geozero::error::{GeozeroError, Result};
use geozero::FeatureProcessor;

/// FlatGeobuf dataset HTTP reader
pub struct HttpFgbReader {
    client: BufferedHttpClient,
    /// Current read offset
    pos: usize,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// File offset of feature section base
    feature_base: usize,
    /// Number of selected features
    count: usize,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Current position in item_filter
    feat_no: usize,
}

impl HttpFgbReader {
    pub async fn open(url: &str) -> Result<HttpFgbReader> {
        let mut client = BufferedHttpClient::new(&url);
        let min_req_size = 512;
        let bytes = client.get(0, 8, min_req_size).await?;
        if bytes != MAGIC_BYTES {
            return Err(GeozeroError::GeometryFormat);
        }
        let bytes = client.get(8, 12, min_req_size).await?;
        let header_size = LittleEndian::read_u32(bytes) as usize;
        let bytes = client.get(12, header_size, min_req_size).await?;
        let header_buf = bytes.to_vec();

        Ok(HttpFgbReader {
            client,
            pos: 0,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
            count: 0,
            feature_base: 0,
            item_filter: None,
            feat_no: 0,
        })
    }
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    fn header_len(&self) -> usize {
        12 + self.fbs.header_buf.len()
    }
    /// Select all features.  Returns feature count.
    pub async fn select_all(&mut self) -> Result<usize> {
        let header = self.fbs.header();
        let count = header.features_count() as usize;
        let index_size = PackedRTree::index_size(count, header.index_node_size());
        // Skip index
        self.feature_base = self.header_len() + index_size;
        self.pos = self.feature_base;
        self.count = count;
        Ok(count)
    }
    /// Select features within a bounding box. Returns count of selected features.
    pub async fn select_bbox(
        &mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<usize> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        let count = header.features_count() as usize;
        let header_len = self.header_len();
        let list = PackedRTree::http_stream_search(
            &mut self.client,
            header_len,
            count,
            PackedRTree::DEFAULT_NODE_SIZE,
            min_x,
            min_y,
            max_x,
            max_y,
        )
        .await?;
        let index_size = PackedRTree::index_size(count, header.index_node_size());
        self.feature_base = self.header_len() + index_size;
        self.pos = self.feature_base;
        self.count = list.len();
        self.item_filter = Some(list);
        Ok(self.count)
    }
    /// Number of selected features
    pub fn features_count(&self) -> usize {
        self.count
    }
    /// Read next feature
    pub async fn next(&mut self) -> Result<Option<&FgbFeature>> {
        let min_req_size = 1_048_576; // 1MB
        if self.feat_no >= self.count {
            return Ok(None);
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            self.pos = self.feature_base + item.offset;
        }
        self.feat_no += 1;
        let bytes = self.client.get(self.pos, 4, min_req_size).await?;
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(bytes) as usize;
        let bytes = self
            .client
            .get(self.pos, feature_size, min_req_size)
            .await?;
        self.fbs.feature_buf = bytes.to_vec(); // Not zero-copy
        self.pos += feature_size;
        Ok(Some(&self.fbs))
    }
    /// Return current feature
    pub fn cur_feature(&self) -> &FgbFeature {
        &self.fbs
    }
    /// Read and process all selected features
    pub async fn process_features<W: FeatureProcessor>(&mut self, out: &mut W) -> Result<()> {
        out.dataset_begin(self.fbs.header().name())?;
        let mut cnt = 0;
        while let Some(feature) = self.next().await? {
            feature.process(out, cnt)?;
            cnt += 1;
        }
        out.dataset_end()
    }
}

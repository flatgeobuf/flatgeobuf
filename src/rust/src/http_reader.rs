use crate::header_generated::flat_geobuf::*;
use crate::http_client::BufferedHttpRangeClient;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{NodeItem, HEADER_MAX_BUFFER_SIZE, MAGIC_BYTES};
use byteorder::{ByteOrder, LittleEndian};
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureAccess, FeatureProcessor};

/// FlatGeobuf dataset HTTP reader
pub struct HttpFgbReader {
    client: BufferedHttpRangeClient,
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
        trace!("starting: opening http reader, reading header");
        let mut client = BufferedHttpRangeClient::new(&url);

        // Because we use a buffered HTTP reader, anything extra we fetch here can
        // be utilized to skip subsequent fetches.
        // Immediately following the header is the optional spatial index, we deliberately fetch
        // a small part of that to skip subsequent requests
        let prefetch_index_bytes: usize = {
            // The actual branching factor will be in the header, but since we don't have the header
            // yet we guess. The consequence of getting this wrong isn't catastrophic, it just means
            // we may be fetching slightly more than we need or that we make an extra request later.
            let assumed_branching_factor = PackedRTree::DEFAULT_NODE_SIZE as usize;

            // NOTE: each layer is exponentially larger
            let prefetched_layers: u32 = 3;

            (0..prefetched_layers)
                .map(|i| assumed_branching_factor.pow(i) * std::mem::size_of::<NodeItem>() as usize)
                .sum()
        };

        // In reality, the header is probably less than half this size, but better to overshoot and
        // fetch an extra kb rather than have to issue a second request.
        let assumed_header_size = 2024;
        let min_req_size = assumed_header_size + prefetch_index_bytes;
        debug!("fetching header. min_req_size: {} (assumed_header_size: {}, prefetched_index_bytes: {})", min_req_size, assumed_header_size, prefetch_index_bytes);

        let bytes = client.get_range(0, 8, min_req_size).await?;
        if bytes != MAGIC_BYTES {
            return Err(GeozeroError::GeometryFormat);
        }
        let bytes = client.get_range(8, 4, min_req_size).await?;
        let header_size = LittleEndian::read_u32(bytes) as usize;
        if header_size > HEADER_MAX_BUFFER_SIZE || header_size < 8 {
            // minimum size check avoids panic in FlatBuffers header decoding
            return Err(GeozeroError::GeometryFormat);
        }
        let bytes = client.get_range(12, header_size, min_req_size).await?;
        let header_buf = bytes.to_vec();

        trace!("completed: opening http reader");
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
        trace!("starting: select_bbox, traversing index");
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
        trace!("completed: select_bbox");
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
        let bytes = self.client.get_range(self.pos, 4, min_req_size).await?;
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(bytes) as usize;
        let bytes = self
            .client
            .get_range(self.pos, feature_size, min_req_size)
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

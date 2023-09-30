use crate::feature_generated::*;
use crate::header_generated::*;
use crate::packed_r_tree::{self, NodeItem, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use crate::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use http_range_client::BufferedHttpRangeClient;

/// FlatGeobuf dataset HTTP reader
pub struct HttpFgbReader {
    client: BufferedHttpRangeClient,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
}

pub struct AsyncFeatureIter {
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
        let mut client = BufferedHttpRangeClient::new(url);

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
                .map(|i| assumed_branching_factor.pow(i) * std::mem::size_of::<NodeItem>())
                .sum()
        };

        // In reality, the header is probably less than half this size, but better to overshoot and
        // fetch an extra kb rather than have to issue a second request.
        let assumed_header_size = 2024;
        let min_req_size = assumed_header_size + prefetch_index_bytes;
        client.set_min_req_size(min_req_size);
        debug!("fetching header. min_req_size: {min_req_size} (assumed_header_size: {assumed_header_size}, prefetched_index_bytes: {prefetch_index_bytes})");

        let bytes = client.get_range(0, 8).await?;
        if !check_magic_bytes(bytes) {
            return Err(Error::MissingMagicBytes);
        }
        let mut bytes = BytesMut::from(client.get_range(8, 4).await?);
        let header_size = LittleEndian::read_u32(&bytes) as usize;
        if header_size > HEADER_MAX_BUFFER_SIZE || header_size < 8 {
            // minimum size check avoids panic in FlatBuffers header decoding
            return Err(Error::IllegalHeaderSize(header_size));
        }
        bytes.put(client.get_range(12, header_size).await?);
        let header_buf = bytes.to_vec();

        // verify flatbuffer
        let _header = size_prefixed_root_as_header(&header_buf)?;

        trace!("completed: opening http reader");
        Ok(HttpFgbReader {
            client,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
        })
    }

    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    fn header_len(&self) -> usize {
        8 + self.fbs.header_buf.len()
    }
    /// Select all features.
    pub async fn select_all(self) -> Result<AsyncFeatureIter> {
        let header = self.fbs.header();
        let count = header.features_count() as usize;
        // TODO: support reading with unknown feature count
        let index_size = if header.index_node_size() > 0 {
            PackedRTree::index_size(count, header.index_node_size())
        } else {
            0
        };
        // Skip index
        let feature_base = self.header_len() + index_size;
        Ok(AsyncFeatureIter {
            client: self.client,
            pos: feature_base,
            fbs: self.fbs,
            count,
            feature_base,
            item_filter: None,
            feat_no: 0,
        })
    }
    /// Select features within a bounding box.
    pub async fn select_bbox(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<AsyncFeatureIter> {
        trace!("starting: select_bbox, traversing index");
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(Error::NoIndex);
        }
        let count = header.features_count() as usize;
        let header_len = self.header_len();
        let mut list = PackedRTree::http_stream_search(
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
        list.sort_by(|a, b| a.offset.cmp(&b.offset));
        let index_size = PackedRTree::index_size(count, header.index_node_size());
        let feature_base = self.header_len() + index_size;
        let count = list.len();
        trace!("completed: select_bbox");
        Ok(AsyncFeatureIter {
            client: self.client,
            pos: feature_base,
            fbs: self.fbs,
            count,
            feature_base,
            item_filter: Some(list),
            feat_no: 0,
        })
    }
}

impl AsyncFeatureIter {
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Number of selected features (might be unknown)
    pub fn features_count(&self) -> Option<usize> {
        if self.count > 0 {
            Some(self.count)
        } else {
            None
        }
    }
    /// Read next feature
    pub async fn next(&mut self) -> Result<Option<&FgbFeature>> {
        let min_req_size = 1_048_576; // 1MB
        self.client.min_req_size(min_req_size);
        if self.feat_no >= self.count {
            return Ok(None);
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            self.pos = self.feature_base + item.offset;
        }
        self.feat_no += 1;
        let mut bytes = BytesMut::from(self.client.get_range(self.pos, 4).await?);
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(&bytes) as usize;
        bytes.put(self.client.get_range(self.pos, feature_size).await?);
        self.fbs.feature_buf = bytes.to_vec(); // Not zero-copy
                                               // verify flatbuffer
        let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)?;
        self.pos += feature_size;
        Ok(Some(&self.fbs))
    }
    /// Return current feature
    pub fn cur_feature(&self) -> &FgbFeature {
        &self.fbs
    }
}

mod geozero_api {
    use crate::AsyncFeatureIter;
    use geozero::{error::Result, FeatureAccess, FeatureProcessor};

    impl AsyncFeatureIter {
        /// Read and process all selected features
        pub async fn process_features<W: FeatureProcessor>(&mut self, out: &mut W) -> Result<()> {
            out.dataset_begin(self.fbs.header().name())?;
            let mut cnt = 0;
            while let Some(feature) = self
                .next()
                .await
                .map_err(|e| geozero::error::GeozeroError::Feature(e.to_string()))?
            {
                feature.process(out, cnt)?;
                cnt += 1;
            }
            out.dataset_end()
        }
    }
}

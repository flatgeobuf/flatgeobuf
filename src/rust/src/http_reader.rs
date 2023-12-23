use crate::feature_generated::*;
use crate::header_generated::*;
use crate::packed_r_tree::{HttpRange, HttpSearchResultItem, NodeItem, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use crate::{Error, Result};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, Bytes, BytesMut};
use http_range_client::BufferedHttpRangeClient;
use std::ops::Range;

// The largest request we'll speculatively make.
// If a single huge feature requires, we'll necessarily exceed this limit.
const DEFAULT_HTTP_FETCH_SIZE: usize = 1_048_576; // 1MB

/// FlatGeobuf dataset HTTP reader
pub struct HttpFgbReader {
    client: BufferedHttpRangeClient,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
}

pub struct AsyncFeatureIter {
    client: BufferedHttpRangeClient,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// Which features to iterate
    selection: FeatureSelection,
    /// Number of selected features
    count: usize,
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
        let count = header.features_count();
        // TODO: support reading with unknown feature count
        let index_size = if header.index_node_size() > 0 {
            PackedRTree::index_size(count as usize, header.index_node_size())
        } else {
            0
        };
        // Skip index
        let feature_base = self.header_len() + index_size;
        Ok(AsyncFeatureIter {
            client: self.client,
            fbs: self.fbs,
            selection: FeatureSelection::SelectAll(SelectAll {
                features_left: count,
                pos: feature_base,
            }),
            count: count as usize,
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

        // request up to this many extra bytes if it means we can eliminate an extra request
        let combine_request_threshold = 256 * 1024;

        let list = PackedRTree::http_stream_search(
            &mut self.client,
            header_len,
            count,
            PackedRTree::DEFAULT_NODE_SIZE,
            min_x,
            min_y,
            max_x,
            max_y,
            combine_request_threshold,
        )
        .await?;
        debug_assert!(
            list.windows(2)
                .all(|w| w[0].range.start() < w[1].range.start()),
            "Since the tree is traversed breadth first, list should be sorted by construction."
        );

        let count = list.len();
        let feature_batches = FeatureBatch::make_batches(list, combine_request_threshold).await?;
        let selection = FeatureSelection::SelectBbox(SelectBbox { feature_batches });
        trace!("completed: select_bbox");
        Ok(AsyncFeatureIter {
            client: self.client,
            fbs: self.fbs,
            selection,
            count,
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
        let Some(buffer) = self.selection.next_feature_buffer(&mut self.client).await? else {
            return Ok(None);
        };

        // Not zero-copy
        self.fbs.feature_buf = buffer.to_vec();
        // verify flatbuffer
        let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)?;
        Ok(Some(&self.fbs))
    }
    /// Return current feature
    pub fn cur_feature(&self) -> &FgbFeature {
        &self.fbs
    }
}

enum FeatureSelection {
    SelectAll(SelectAll),
    SelectBbox(SelectBbox),
}

impl FeatureSelection {
    async fn next_feature_buffer(
        &mut self,
        client: &mut BufferedHttpRangeClient,
    ) -> Result<Option<Bytes>> {
        match self {
            FeatureSelection::SelectAll(select_all) => select_all.next_buffer(client).await,
            FeatureSelection::SelectBbox(select_bbox) => select_bbox.next_buffer(client).await,
        }
    }
}

struct SelectAll {
    /// Features left
    features_left: u64,

    /// How many bytes into the file we've read so far
    pos: usize,
}

impl SelectAll {
    async fn next_buffer(&mut self, client: &mut BufferedHttpRangeClient) -> Result<Option<Bytes>> {
        client.min_req_size(DEFAULT_HTTP_FETCH_SIZE);

        if self.features_left == 0 {
            return Ok(None);
        }
        self.features_left -= 1;

        let mut feature_buffer = BytesMut::from(client.get_range(self.pos, 4).await?);
        self.pos += 4;
        let feature_size = LittleEndian::read_u32(&feature_buffer) as usize;
        feature_buffer.put(client.get_range(self.pos, feature_size).await?);
        self.pos += feature_size;

        Ok(Some(feature_buffer.freeze()))
    }
}

struct SelectBbox {
    /// Selected features
    feature_batches: Vec<FeatureBatch>,
}

impl SelectBbox {
    async fn next_buffer(&mut self, client: &mut BufferedHttpRangeClient) -> Result<Option<Bytes>> {
        let mut next_buffer = None;
        while next_buffer.is_none() {
            let Some(feature_batch) = self.feature_batches.last_mut() else {
                break;
            };
            let Some(buffer) = feature_batch.next_buffer(client).await? else {
                // done with this batch
                self.feature_batches
                    .pop()
                    .expect("already asserted feature_batches was non-empty");
                continue;
            };
            next_buffer = Some(buffer)
        }

        Ok(next_buffer)
    }
}

struct FeatureBatch {
    /// The byte location of each feature within the file
    feature_ranges: std::vec::IntoIter<HttpRange>,

    /// When fetching new data, how many bytes should we fetch at once.
    /// It was computed based on the specific feature ranges of the batch
    /// to optimize number of requests vs. wasted bytes vs. resident memory
    min_request_size: usize,
}

impl FeatureBatch {
    async fn make_batches(
        feature_ranges: Vec<HttpSearchResultItem>,
        combine_request_threshold: usize,
    ) -> Result<Vec<Self>> {
        let mut batched_ranges = vec![];

        for search_result_item in feature_ranges.into_iter() {
            let Some(latest_batch) = batched_ranges.last_mut() else {
                batched_ranges.push(vec![search_result_item.range]);
                continue;
            };

            let previous_item = latest_batch.last().expect("we never push an empty batch");

            let HttpRange::Range(Range { end: prev_end, .. }) = previous_item else {
                debug_assert!(false, "This shouldn't happen. Only the very last feature is expected to have an unknown length");
                batched_ranges.push(vec![search_result_item.range]);
                continue;
            };

            let wasted_bytes = search_result_item.range.start() - prev_end;
            if wasted_bytes < combine_request_threshold {
                if wasted_bytes == 0 {
                    trace!("adjacent feature");
                } else {
                    trace!("wasting {wasted_bytes} to avoid an extra request");
                }
                latest_batch.push(search_result_item.range)
            } else {
                trace!("creating a new request for batch rather than wasting {wasted_bytes} bytes");
                batched_ranges.push(vec![search_result_item.range]);
            }
        }

        let mut batches: Vec<_> = batched_ranges.into_iter().map(FeatureBatch::new).collect();
        batches.reverse();
        Ok(batches)
    }

    fn new(feature_ranges: Vec<HttpRange>) -> Self {
        let first = feature_ranges
            .first()
            .expect("We never create empty batches");
        let last = feature_ranges
            .last()
            .expect("We never create empty batches");

        // `last.length()` should only be None if this batch includes the final feature
        // in the dataset. Since we can't infer its actual length, we'll fetch only
        // the first 4 bytes of that feature buffer, which will tell us the actual length
        // of the feature buffer for the subsequent request.
        let last_feature_length = last.length().unwrap_or(4);

        let covering_range = first.start()..last.start() + last_feature_length;

        let min_request_size = covering_range
            .len()
            // Since it's all held in memory, don't fetch more than DEFAULT_HTTP_FETCH_SIZE at a time
            // unless necessary.
            .min(DEFAULT_HTTP_FETCH_SIZE);

        Self {
            feature_ranges: feature_ranges.into_iter(),
            min_request_size,
        }
    }

    async fn next_buffer(&mut self, client: &mut BufferedHttpRangeClient) -> Result<Option<Bytes>> {
        client.set_min_req_size(self.min_request_size);
        let Some(feature_range) = self.feature_ranges.next() else {
            return Ok(None);
        };
        // Only set min_request_size for the first request.
        //
        // This should only affect a batch that contains the final feature, otherwise
        // we've calculated `batchSize` to get all the data we need for the batch.
        // For the very final feature in a dataset, we don't know it's length, so we
        // will end up executing an extra request for that batch.
        self.min_request_size = 0;

        let mut pos = feature_range.start();
        let mut feature_buffer = BytesMut::from(client.get_range(pos, 4).await?);
        pos += 4;
        let feature_size = LittleEndian::read_u32(&feature_buffer) as usize;
        feature_buffer.put(client.get_range(pos, feature_size).await?);

        Ok(Some(feature_buffer.freeze()))
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

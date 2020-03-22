use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::MAGIC_BYTES;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};

/// FlatGeobuf header reader
pub struct HeaderReader {
    header_buf: Vec<u8>,
}

impl HeaderReader {
    pub fn read<R: Read + Seek>(mut reader: R) -> std::result::Result<Self, std::io::Error> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if magic_buf != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf);

        let mut data = HeaderReader {
            header_buf: Vec::with_capacity(header_size as usize),
        };
        data.header_buf.resize(header_size as usize, 0);
        reader.read_exact(&mut data.header_buf)?;

        Ok(data)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.header_buf[..])
    }
}

/// FlatGeobuf feature reader
pub struct FeatureReader {
    feature_base: u64,
    feature_buf: Vec<u8>,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Current position in item_filter
    filter_idx: usize,
}

impl FeatureReader {
    /// Skip R-Tree index
    pub fn select_all<R: Read + Seek>(
        mut reader: R,
        header: &Header,
    ) -> std::result::Result<Self, std::io::Error> {
        let mut data = FeatureReader {
            feature_base: 0,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        };
        // Skip index
        let index_size =
            PackedRTree::index_size(header.features_count() as usize, header.index_node_size());
        data.feature_base = reader.seek(SeekFrom::Current(index_size as i64))?;
        Ok(data)
    }
    /// Read R-Tree index and build filter for features within bbox
    pub fn select_bbox<R: Read + Seek>(
        mut reader: R,
        header: &Header,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> std::result::Result<Self, std::io::Error> {
        let mut data = FeatureReader {
            feature_base: 0,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        };
        let tree = PackedRTree::from_buf(
            &mut reader,
            header.features_count() as usize,
            PackedRTree::DEFAULT_NODE_SIZE,
        );
        let mut list = tree.search(min_x, min_y, max_x, max_y);
        list.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        data.item_filter = Some(list);
        data.feature_base = reader.seek(SeekFrom::Current(0))?;
        Ok(data)
    }
    /// Number of selected features
    pub fn filter_count(&self) -> Option<usize> {
        self.item_filter.as_ref().map(|f| f.len())
    }
    /// Read next feature
    pub fn next<R: Read + Seek>(
        &mut self,
        mut reader: R,
    ) -> std::result::Result<Feature, std::io::Error> {
        // impl Iterator for Reader is diffcult, because of Feature lifetime
        if let Some(filter) = &self.item_filter {
            if self.filter_idx >= filter.len() {
                return Err(Error::new(ErrorKind::Other, "No more features"));
            }
            let item = &filter[self.filter_idx];
            reader.seek(SeekFrom::Start(self.feature_base + item.offset as u64))?;
            self.filter_idx += 1;
        }
        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let feature_size = u32::from_le_bytes(size_buf);
        self.feature_buf.resize(feature_size as usize, 0);
        reader.read_exact(&mut self.feature_buf)?;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
    }
    /// Return current feature
    pub fn cur_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
}

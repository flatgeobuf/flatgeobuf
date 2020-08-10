use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::MAGIC_BYTES;
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureProcessor, ReadSeek};
use std::io::SeekFrom;

/// FlatGeobuf dataset reader
pub struct FgbReader<'a> {
    reader: &'a mut dyn ReadSeek,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// File offset of feature section base
    feature_base: u64,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Number of selected features
    count: usize,
    /// Current feature number
    feat_no: usize,
}

impl<'a> FgbReader<'a> {
    /// Open dataset by reading the header information
    pub fn open<R: 'a + ReadSeek>(reader: &'a mut R) -> Result<Self> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if magic_buf != MAGIC_BYTES {
            return Err(GeozeroError::GeometryFormat);
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf) as usize;

        let mut header_buf = Vec::with_capacity(header_size);
        header_buf.resize(header_size, 0);
        reader.read_exact(&mut header_buf)?;

        Ok(FgbReader {
            reader,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
            feature_base: 0,
            item_filter: None,
            count: 0,
            feat_no: 0,
        })
    }
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Select all features.  Returns feature count.
    pub fn select_all(&mut self) -> Result<usize> {
        let header = self.fbs.header();
        let count = header.features_count() as usize;
        let index_size = PackedRTree::index_size(count, header.index_node_size());
        // Skip index
        self.feature_base = self.reader.seek(SeekFrom::Current(index_size as i64))?;
        self.count = count;
        Ok(count)
    }
    /// Select features within a bounding box. Returns count of selected features.
    pub fn select_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<usize> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        let list = PackedRTree::stream_search(
            &mut self.reader,
            header.features_count() as usize,
            PackedRTree::DEFAULT_NODE_SIZE,
            min_x,
            min_y,
            max_x,
            max_y,
        )?;
        self.feature_base = self.reader.seek(SeekFrom::Current(0))?;
        self.count = list.len();
        self.item_filter = Some(list);
        Ok(self.count)
    }
    /// Number of selected features
    pub fn features_count(&self) -> usize {
        self.count
    }
    /// Read next feature
    pub fn next(&mut self) -> Result<Option<&FgbFeature>> {
        if self.feat_no >= self.count {
            return Ok(None);
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            self.reader
                .seek(SeekFrom::Start(self.feature_base + item.offset as u64))?;
        }
        self.feat_no += 1;
        let mut size_buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut size_buf)?;
        let feature_size = u32::from_le_bytes(size_buf);
        self.fbs.feature_buf.resize(feature_size as usize, 0);
        self.reader.read_exact(&mut self.fbs.feature_buf)?;
        Ok(Some(&self.fbs))
    }
    /// Return current feature
    pub fn cur_feature(&self) -> &FgbFeature {
        &self.fbs
    }
    /// Read and process all selected features
    pub fn process_features<W: FeatureProcessor>(&mut self, out: &mut W) -> Result<()> {
        out.dataset_begin(self.fbs.header().name())?;
        let mut cnt = 0;
        while let Some(feature) = self.next()? {
            feature.process(out, cnt)?;
            cnt += 1;
        }
        out.dataset_end()
    }
}

mod inspect {
    use super::*;

    impl FgbReader<'_> {
        /// Process R-Tree index for debugging purposes
        #[doc(hidden)]
        pub fn process_index<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
            let features_count = self.header().features_count() as usize;
            let index_node_size = self.header().index_node_size();
            let index = PackedRTree::from_buf(&mut self.reader, features_count, index_node_size)?;
            index.process_index(processor)
        }
    }

    #[test]
    fn dump_index() -> Result<()> {
        use geozero_core::geojson::GeoJsonWriter;
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
        let mut fgb = FgbReader::open(&mut filein)?;
        let mut fout = BufWriter::new(File::create("/tmp/countries-index.json")?);

        fgb.process_index(&mut GeoJsonWriter::new(&mut fout))?;
        Ok(())
    }
}

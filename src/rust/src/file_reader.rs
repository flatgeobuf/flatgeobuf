use crate::feature_generated::*;
use crate::header_generated::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use fallible_streaming_iterator::FallibleStreamingIterator;
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureAccess, FeatureProcessor, GeozeroDatasource};
use std::io::{Read, Seek, SeekFrom};

/// FlatGeobuf dataset reader
pub struct FgbReader<'a, R: Read + Seek> {
    reader: &'a mut R,
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

impl<'a, R: Read + Seek> FgbReader<'a, R> {
    /// Open dataset by reading the header information
    pub fn open(reader: &'a mut R) -> Result<Self> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if !check_magic_bytes(&magic_buf) {
            return Err(GeozeroError::GeometryFormat);
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf) as usize;
        if header_size > HEADER_MAX_BUFFER_SIZE || header_size < 8 {
            // minimum size check avoids panic in FlatBuffers header decoding
            return Err(GeozeroError::GeometryFormat);
        }
        let mut header_buf = Vec::with_capacity(header_size + 4);
        header_buf.extend_from_slice(&size_buf);
        header_buf.resize(header_buf.capacity(), 0);
        reader.read_exact(&mut header_buf[4..])?;

        // verify flatbuffer
        let _header = size_prefixed_root_as_header(&header_buf)
            .map_err(|e| GeozeroError::Geometry(e.to_string()))?;

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

/// `FallibleStreamingIterator` differs from the standard library's `Iterator`
/// in two ways:
/// * each call to `next` can fail.
/// * returned `FgbFeature` is valid until `next` is called again or `FgbReader` is
///   reset or finalized.
///
/// While these iterators cannot be used with Rust `for` loops, `while let`
/// loops offer a similar level of ergonomics:
/// ```rust
/// use flatgeobuf::*;
/// # use std::fs::File;
/// # use std::io::BufReader;
///
/// # fn read_fbg() -> geozero::error::Result<()> {
/// # let mut filein = BufReader::new(File::open("countries.fgb")?);
/// # let mut fgb = FgbReader::open(&mut filein)?;
/// # fgb.select_all()?;
/// while let Some(feature) = fgb.next()? {
///     let props = feature.properties()?;
///     println!("{}", props["name"]);
/// }
/// # Ok(())
/// # }
/// ```
impl<'a, R: Read + Seek> FallibleStreamingIterator for FgbReader<'a, R> {
    type Error = GeozeroError;
    type Item = FgbFeature;

    fn advance(&mut self) -> Result<()> {
        if self.feat_no >= self.count {
            self.feat_no = self.count + 1;
            return Ok(());
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            self.reader
                .seek(SeekFrom::Start(self.feature_base + item.offset as u64))?;
        }
        self.feat_no += 1;
        self.fbs.feature_buf.resize(4, 0);
        self.reader.read_exact(&mut self.fbs.feature_buf)?;
        let sbuf = &self.fbs.feature_buf;
        let feature_size = u32::from_le_bytes([sbuf[0], sbuf[1], sbuf[2], sbuf[3]]) as usize;
        self.fbs.feature_buf.resize(feature_size + 4, 0);
        self.reader.read_exact(&mut self.fbs.feature_buf[4..])?;
        // verify flatbuffer
        let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)
            .map_err(|e| GeozeroError::Geometry(e.to_string()))?;
        Ok(())
    }

    fn get(&self) -> Option<&FgbFeature> {
        if self.feat_no > self.count {
            None
        } else {
            Some(&self.fbs)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.feat_no >= self.count {
            (0, Some(0))
        } else {
            let remaining = self.count - self.feat_no;
            (remaining, Some(remaining))
        }
    }
}

impl<'a, T: Read + Seek> GeozeroDatasource for FgbReader<'a, T> {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.process_features(processor)
    }
}

mod inspect {
    use super::*;

    impl<'a, R: Read + Seek> FgbReader<'a, R> {
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
        use geozero::geojson::GeoJsonWriter;
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
        let mut fgb = FgbReader::open(&mut filein)?;
        let mut fout = BufWriter::new(File::create("/tmp/countries-index.json")?);

        fgb.process_index(&mut GeoJsonWriter::new(&mut fout))?;
        Ok(())
    }
}

use crate::feature_generated::*;
use crate::file_reader::reader_state::*;
use crate::header_generated::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use fallible_streaming_iterator::FallibleStreamingIterator;
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureAccess, FeatureProcessor, GeozeroDatasource};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;

/// FlatGeobuf dataset reader
pub struct FgbReader<'a, R: Read + Seek, State = Initial> {
    reader: &'a mut R,
    /// FlatBuffers verification
    verify: bool,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// File offset of feature section base
    feature_base: u64,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Number of selected features or remaining file size in case of unknown feature count
    size_info: SizeInfo,
    /// Current feature number
    feat_no: usize,
    /// Reader state
    state: PhantomData<State>,
}

#[derive(Debug)]
enum SizeInfo {
    FeatureCount(usize),
    RemainingBytes(u64),
    // Undefined or all features read
    None,
}

// Reader states for ensuring correct read API usage at compile-time
pub(crate) mod reader_state {
    pub struct Initial;
    pub struct Open;
    pub struct FeaturesSelected;
}

impl<'a, R: Read + Seek> FgbReader<'a, R, Initial> {
    /// Open dataset by reading the header information
    pub fn open(reader: &'a mut R) -> Result<FgbReader<'a, R, Open>> {
        Self::read_header(reader, true)
    }
    /// Open dataset by reading the header information without FlatBuffers verification
    pub unsafe fn open_unchecked(reader: &'a mut R) -> Result<FgbReader<'a, R, Open>> {
        Self::read_header(reader, false)
    }
    fn read_header(reader: &'a mut R, verify: bool) -> Result<FgbReader<'a, R, Open>> {
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

        if verify {
            let _header = size_prefixed_root_as_header(&header_buf)
                .map_err(|e| GeozeroError::Geometry(e.to_string()))?;
        };

        Ok(FgbReader {
            reader,
            verify,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
            feature_base: 0,
            item_filter: None,
            size_info: SizeInfo::None,
            feat_no: 0,
            state: PhantomData::<Open>,
        })
    }
}

impl<'a, R: Read + Seek> FgbReader<'a, R, Open> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Select all features.
    pub fn select_all(self) -> Result<FgbReader<'a, R, FeaturesSelected>> {
        let header = self.fbs.header();
        let count = header.features_count() as usize;
        let index_size = if header.index_node_size() > 0 && count > 0 {
            PackedRTree::index_size(count, header.index_node_size())
        } else {
            0
        };
        // Skip index
        let feature_base = self.reader.seek(SeekFrom::Current(index_size as i64))?;
        let size_info = if count > 0 {
            SizeInfo::FeatureCount(count)
        } else {
            // if dataset contains features, feature count is undefined
            let end = self.reader.seek(SeekFrom::End(0))?;
            let _ = self.reader.seek(SeekFrom::Start(feature_base))?;
            let remaining_bytes = end - feature_base;
            if remaining_bytes > 0 {
                SizeInfo::RemainingBytes(remaining_bytes)
            } else {
                SizeInfo::FeatureCount(0)
            }
        };
        Ok(FgbReader {
            reader: self.reader,
            verify: self.verify,
            fbs: self.fbs,
            feature_base,
            item_filter: None,
            size_info,
            feat_no: 0,
            state: PhantomData::<FeaturesSelected>,
        })
    }
    /// Select features within a bounding box.
    pub fn select_bbox(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<FgbReader<'a, R, FeaturesSelected>> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(GeozeroError::Geometry("Index missing".to_string()));
        }
        let list = PackedRTree::stream_search(
            &mut self.reader,
            header.features_count() as usize,
            PackedRTree::DEFAULT_NODE_SIZE,
            min_x,
            min_y,
            max_x,
            max_y,
        )?;
        let feature_base = self.reader.seek(SeekFrom::Current(0))?;
        let size_info = SizeInfo::FeatureCount(list.len());
        Ok(FgbReader {
            reader: self.reader,
            verify: self.verify,
            fbs: self.fbs,
            feature_base,
            item_filter: Some(list),
            size_info,
            feat_no: 0,
            state: PhantomData::<FeaturesSelected>,
        })
    }
}

impl<'a, R: Read + Seek> FgbReader<'a, R, FeaturesSelected> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Number of selected features (might be unknown)
    pub fn features_count(&self) -> Option<usize> {
        match self.size_info {
            SizeInfo::FeatureCount(count) => Some(count),
            _ => None,
        }
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
/// # let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
/// while let Some(feature) = fgb.next()? {
///     let props = feature.properties()?;
///     println!("{}", props["name"]);
/// }
/// # Ok(())
/// # }
/// ```
impl<'a, R: Read + Seek> FallibleStreamingIterator for FgbReader<'a, R, FeaturesSelected> {
    type Error = GeozeroError;
    type Item = FgbFeature;

    fn advance(&mut self) -> Result<()> {
        match self.size_info {
            SizeInfo::FeatureCount(count) => {
                if self.feat_no >= count {
                    self.size_info = SizeInfo::None;
                    return Ok(());
                }
            }
            SizeInfo::RemainingBytes(bytes) => {
                if bytes == 0 {
                    self.size_info = SizeInfo::None;
                    return Ok(());
                }
            }
            SizeInfo::None => {
                return Ok(());
            }
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
        if self.verify {
            let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)
                .map_err(|e| GeozeroError::Geometry(e.to_string()))?;
        }
        if let SizeInfo::RemainingBytes(bytes) = self.size_info {
            self.size_info = SizeInfo::RemainingBytes(bytes - self.fbs.feature_buf.len() as u64);
        }
        Ok(())
    }

    fn get(&self) -> Option<&FgbFeature> {
        match self.size_info {
            SizeInfo::FeatureCount(_) => Some(&self.fbs),
            SizeInfo::RemainingBytes(_) => Some(&self.fbs),
            SizeInfo::None => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.size_info {
            SizeInfo::FeatureCount(count) => {
                if self.feat_no >= count {
                    (0, Some(0))
                } else {
                    let remaining = count - self.feat_no;
                    (remaining, Some(remaining))
                }
            }
            SizeInfo::RemainingBytes(_) => (0, None),
            SizeInfo::None => (0, Some(0)),
        }
    }
}

impl<'a, T: Read + Seek> GeozeroDatasource for FgbReader<'a, T, FeaturesSelected> {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.process_features(processor)
    }
}

mod inspect {
    use super::*;

    impl<'a, R: Read + Seek> FgbReader<'a, R, Open> {
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

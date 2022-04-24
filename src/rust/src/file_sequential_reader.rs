use crate::feature_generated::*;
use crate::header_generated::*;
use crate::packed_r_tree::PackedRTree;
use crate::properties_reader::FgbFeature;
use crate::reader_state::*;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use fallible_streaming_iterator::FallibleStreamingIterator;
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureAccess, FeatureProcessor, GeozeroDatasource};
use std::io::Read;
use std::marker::PhantomData;

/// FlatGeobuf sequential dataset reader
pub struct FgbSequentialReader<'a, R: Read, State = Initial> {
    reader: &'a mut R,
    /// FlatBuffers verification
    verify: bool,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// Number of selected features (None for undefined feature count)
    count: Option<usize>,
    /// File offset within feature section
    cur_pos: u64,
    /// All features read or end of file reached
    finished: bool,
    /// Reader state
    state: PhantomData<State>,
}

impl<'a, R: Read> FgbSequentialReader<'a, R, Initial> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Open dataset by reading the header information
    pub fn open(reader: &'a mut R) -> Result<FgbSequentialReader<'a, R, Open>> {
        Self::read_header(reader, true)
    }
    /// Open dataset by reading the header information without FlatBuffers verification
    pub unsafe fn open_unchecked(reader: &'a mut R) -> Result<FgbSequentialReader<'a, R, Open>> {
        Self::read_header(reader, false)
    }
    fn read_header(reader: &'a mut R, verify: bool) -> Result<FgbSequentialReader<'a, R, Open>> {
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
        }

        // let feature_base: u64 = (header_size + MAGIC_BYTES.len() + index_size) as u64;

        Ok(FgbSequentialReader {
            reader,
            verify,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
            count: None,
            cur_pos: 0,
            finished: false,
            state: PhantomData::<Open>,
        })
    }
}

impl<'a, R: Read> FgbSequentialReader<'a, R, Open> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Select all features.
    pub fn select_all(mut self) -> Result<FgbSequentialReader<'a, R, FeaturesSelected>> {
        let header = self.fbs.header();
        let feat_count = header.features_count() as usize;
        let index_size = if header.index_node_size() > 0 && feat_count > 0 {
            PackedRTree::index_size(feat_count, header.index_node_size())
        } else {
            0
        };
        std::io::copy(
            &mut self.reader.take(index_size as u64),
            &mut std::io::sink(),
        )?;
        // Detect empty dataset by reading the first feature size
        self.fbs.feature_buf.resize(4, 0);
        let finished = self.reader.read_exact(&mut self.fbs.feature_buf).is_err();
        let count = if feat_count > 0 {
            Some(feat_count)
        } else if finished {
            Some(0)
        } else {
            None
        };
        Ok(FgbSequentialReader {
            reader: self.reader,
            verify: self.verify,
            fbs: self.fbs,
            count,
            cur_pos: 4,
            finished,
            state: PhantomData::<FeaturesSelected>,
        })
    }
}

impl<'a, R: Read> FgbSequentialReader<'a, R, FeaturesSelected> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }
    /// Number of selected features (might be unknown)
    pub fn features_count(&self) -> Option<usize> {
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
/// # let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
/// while let Some(feature) = fgb.next()? {
///     let props = feature.properties()?;
///     println!("{}", props["name"]);
/// }
/// # Ok(())
/// # }
/// ```
impl<'a, R: Read> FallibleStreamingIterator for FgbSequentialReader<'a, R, FeaturesSelected> {
    type Error = GeozeroError;
    type Item = FgbFeature;

    fn advance(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        if self.cur_pos != 4 {
            // First featare size is read in select_all or select_bbox
            self.fbs.feature_buf.resize(4, 0);
            if self.reader.read_exact(&mut self.fbs.feature_buf).is_err() {
                self.finished = true;
                return Ok(());
            }
        }
        let sbuf = &self.fbs.feature_buf;
        let feature_size = u32::from_le_bytes([sbuf[0], sbuf[1], sbuf[2], sbuf[3]]) as usize;
        self.fbs.feature_buf.resize(feature_size + 4, 0);
        self.reader.read_exact(&mut self.fbs.feature_buf[4..])?;
        if self.verify {
            let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)
                .map_err(|e| GeozeroError::Geometry(e.to_string()))?;
        }
        self.cur_pos += self.fbs.feature_buf.len() as u64;
        Ok(())
    }

    fn get(&self) -> Option<&FgbFeature> {
        if self.finished {
            None
        } else {
            Some(&self.fbs)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None) // Maybe we could estimate?
    }
}

impl<'a, T: Read> GeozeroDatasource for FgbSequentialReader<'a, T, FeaturesSelected> {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.process_features(processor)
    }
}

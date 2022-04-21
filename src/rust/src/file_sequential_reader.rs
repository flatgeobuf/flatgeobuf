use crate::feature_generated::*;
use crate::file_reader::reader_state::*;
use crate::header_generated::*;
use crate::packed_r_tree::PackedRTree;
use crate::properties_reader::FgbFeature;
use crate::MAGIC_BYTES;
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
    /// Index size
    index_size: usize,
    /// File offset of feature section base
    feature_base: u64,
    /// Number of selected features
    count: usize,
    /// Current feature number
    feat_no: usize,
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
    pub fn open_unchecked(reader: &'a mut R) -> Result<FgbSequentialReader<'a, R, Open>> {
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

        let header: Header;
        if verify {
            header = size_prefixed_root_as_header(&header_buf)
                .map_err(|e| GeozeroError::Geometry(e.to_string()))?;
        } else {
            header = size_prefixed_root_as_header_unchecked(&header_buf);
        }

        let count = header.features_count() as usize;
        let index_size = if header.index_node_size() > 0 {
            PackedRTree::index_size(count, header.index_node_size())
        } else {
            0
        };

        let feature_base: u64 = (header_size + MAGIC_BYTES.len() + index_size) as u64;

        Ok(FgbSequentialReader {
            reader,
            verify,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
            index_size,
            feature_base,
            count,
            feat_no: 0,
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
    pub fn select_all(self) -> Result<FgbSequentialReader<'a, R, FeaturesSelected>> {
        std::io::copy(
            &mut self.reader.take(self.index_size as u64),
            &mut std::io::sink(),
        )?;
        Ok(FgbSequentialReader {
            reader: self.reader,
            verify: self.verify,
            fbs: self.fbs,
            index_size: self.index_size,
            feature_base: self.feature_base,
            count: self.count,
            feat_no: 0,
            state: PhantomData::<FeaturesSelected>,
        })
    }
}

impl<'a, R: Read> FgbSequentialReader<'a, R, FeaturesSelected> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
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
        if self.feat_no >= self.count {
            self.feat_no = self.count + 1;
            return Ok(());
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

impl<'a, T: Read> GeozeroDatasource for FgbSequentialReader<'a, T, FeaturesSelected> {
    /// Consume and process all selected features.
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        self.process_features(processor)
    }
}

use crate::feature_generated::*;
use crate::header_generated::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use crate::{Error, Result};
use fallible_streaming_iterator::FallibleStreamingIterator;
use std::io::{self, Read, Seek, SeekFrom};
use std::marker::PhantomData;

/// FlatGeobuf dataset reader
pub struct FgbReader<R> {
    reader: R,
    /// FlatBuffers verification
    verify: bool,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
}

pub struct FeatureIter<R, S> {
    reader: R,
    /// FlatBuffers verification
    verify: bool,
    // feature reading requires header access, therefore
    // header_buf is included in the FgbFeature struct.
    fbs: FgbFeature,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Number of selected features (None for undefined feature count)
    count: Option<usize>,
    /// Current feature number
    feat_no: usize,
    /// File offset within feature section
    cur_pos: u64,
    /// Reading state
    state: State,
    /// Whether or not the underlying reader is Seek
    seekable_marker: PhantomData<S>,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Init,
    ReadFirstFeatureSize,
    Reading,
    Finished,
}

#[doc(hidden)]
pub mod reader_trait {
    pub struct Seekable;
    pub struct NotSeekable;
}
use reader_trait::*;

impl<R: Read> FgbReader<R> {
    /// Open dataset by reading the header information
    pub fn open(reader: R) -> Result<FgbReader<R>> {
        Self::read_header(reader, true)
    }

    /// Open dataset by reading the header information without FlatBuffers verification
    ///
    /// # Safety
    /// This method is unsafe because it does not verify the FlatBuffers header.
    /// It is still safe from the Rust safety guarantees perspective, but it may cause
    /// undefined behavior if the FlatBuffers header is invalid.
    pub unsafe fn open_unchecked(reader: R) -> Result<FgbReader<R>> {
        Self::read_header(reader, false)
    }

    fn read_header(mut reader: R, verify: bool) -> Result<FgbReader<R>> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if !check_magic_bytes(&magic_buf) {
            return Err(Error::MissingMagicBytes);
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf) as usize;
        if header_size > HEADER_MAX_BUFFER_SIZE || header_size < 8 {
            // minimum size check avoids panic in FlatBuffers header decoding
            return Err(Error::IllegalHeaderSize(header_size));
        }
        let mut header_buf = Vec::with_capacity(header_size + 4);
        header_buf.extend_from_slice(&size_buf);
        header_buf.resize(header_buf.capacity(), 0);
        reader.read_exact(&mut header_buf[4..])?;

        if verify {
            let _header = size_prefixed_root_as_header(&header_buf)?;
        }

        Ok(FgbReader {
            reader,
            verify,
            fbs: FgbFeature {
                header_buf,
                feature_buf: Vec::new(),
            },
        })
    }

    /// Select all features without using seek.
    ///
    /// This can be used to read from an input stream.
    pub fn select_all_seq(mut self) -> Result<FeatureIter<R, NotSeekable>> {
        // skip index
        let index_size = self.index_size();
        io::copy(&mut (&mut self.reader).take(index_size), &mut io::sink())?;

        Ok(FeatureIter::new(self.reader, self.verify, self.fbs, None))
    }

    /// Select features within a bounding box without using seek.
    ///
    /// This can be used to read from an input stream.
    pub fn select_bbox_seq(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<FeatureIter<R, NotSeekable>> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(Error::NoIndex);
        }
        let index = PackedRTree::from_buf(
            &mut self.reader,
            header.features_count() as usize,
            header.index_node_size(),
        )?;
        let mut list = index.search(min_x, min_y, max_x, max_y)?;
        // debug_assert!(
        //     list.windows(2).all(|w| w[0].offset < w[1].offset),
        //     "Since the tree is traversed breadth first, list should be sorted by construction."
        // );
        list.sort_by_key(|x| x.offset);
        println!("{:?}", list);
        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.fbs,
            Some(list),
        ))
    }
}

impl<R: Read + Seek> FgbReader<R> {
    /// Select all features.
    pub fn select_all(mut self) -> Result<FeatureIter<R, Seekable>> {
        // skip index
        let index_size = self.index_size();
        self.reader.seek(SeekFrom::Current(index_size as i64))?;

        Ok(FeatureIter::new(self.reader, self.verify, self.fbs, None))
    }
    /// Select features within a bounding box.
    pub fn select_bbox(
        mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<FeatureIter<R, Seekable>> {
        // Read R-Tree index and build filter for features within bbox
        let header = self.fbs.header();
        if header.index_node_size() == 0 || header.features_count() == 0 {
            return Err(Error::NoIndex);
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
        // debug_assert!(
        //     list.windows(2).all(|w| w[0].offset < w[1].offset),
        //     "Since the tree is traversed breadth first, list should be sorted by construction."
        // );

        Ok(FeatureIter::new(
            self.reader,
            self.verify,
            self.fbs,
            Some(list),
        ))
    }
}

impl<R: Read> FgbReader<R> {
    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }

    fn index_size(&self) -> u64 {
        let header = self.fbs.header();
        let feat_count = header.features_count() as usize;
        if header.index_node_size() > 0 && feat_count > 0 {
            PackedRTree::index_size(feat_count, header.index_node_size()) as u64
        } else {
            0
        }
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
/// # fn read_fbg() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// # let mut filein = BufReader::new(File::open("countries.fgb")?);
/// # let mut fgb = FgbReader::open(&mut filein)?.select_all_seq()?;
/// while let Some(feature) = fgb.next()? {
///     let props = feature.properties()?;
///     println!("{}", props["name"]);
/// }
/// # Ok(())
/// # }
/// ```
impl<R: Read> FallibleStreamingIterator for FeatureIter<R, NotSeekable> {
    type Item = FgbFeature;
    type Error = Error;

    fn advance(&mut self) -> Result<()> {
        if self.advance_finished() {
            return Ok(());
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            if item.offset as u64 > self.cur_pos {
            if self.state == State::ReadFirstFeatureSize {
                self.state = State::Reading;
            }
            // skip features
            let seek_bytes = item.offset as u64 - self.cur_pos;
            io::copy(&mut (&mut self.reader).take(seek_bytes), &mut io::sink())?;
            self.cur_pos += seek_bytes;
            }
        }
        self.read_feature()
    }

    fn get(&self) -> Option<&FgbFeature> {
        self.iter_get()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_size_hint()
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
/// # fn read_fbg() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// # let mut filein = BufReader::new(File::open("countries.fgb")?);
/// # let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
/// while let Some(feature) = fgb.next()? {
///     let props = feature.properties()?;
///     println!("{}", props["name"]);
/// }
/// # Ok(())
/// # }
/// ```
impl<R: Read + Seek> FallibleStreamingIterator for FeatureIter<R, Seekable> {
    type Item = FgbFeature;
    type Error = Error;

    fn advance(&mut self) -> Result<()> {
        if self.advance_finished() {
            return Ok(());
        }
        if let Some(filter) = &self.item_filter {
            let item = &filter[self.feat_no];
            if item.offset as u64 > self.cur_pos {
                if self.state == State::ReadFirstFeatureSize {
                    self.state = State::Reading;
                }
                // skip features
                let seek_bytes = item.offset as u64 - self.cur_pos;
                self.reader.seek(SeekFrom::Current(seek_bytes as i64))?;
                self.cur_pos += seek_bytes;
            }
        }
        self.read_feature()
    }

    fn get(&self) -> Option<&FgbFeature> {
        self.iter_get()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_size_hint()
    }
}

mod geozero_api {
    use crate::reader_trait::{NotSeekable, Seekable};
    use crate::{FeatureIter, FgbFeature};
    use fallible_streaming_iterator::FallibleStreamingIterator;
    use geozero::error::GeozeroError;
    use geozero::{FeatureAccess, FeatureProcessor, GeozeroDatasource};
    use std::io::{Read, Seek};

    impl<T: Read> GeozeroDatasource for FeatureIter<T, NotSeekable> {
        /// Consume and process all selected features.
        fn process<P: FeatureProcessor>(
            &mut self,
            processor: &mut P,
        ) -> geozero::error::Result<()> {
            self.process_features(processor)
        }
    }

    impl<T: Read + Seek> GeozeroDatasource for FeatureIter<T, Seekable> {
        /// Consume and process all selected features.
        fn process<P: FeatureProcessor>(
            &mut self,
            processor: &mut P,
        ) -> geozero::error::Result<()> {
            self.process_features(processor)
        }
    }

    impl<R: Read> FeatureIter<R, NotSeekable> {
        /// Return current feature
        pub fn cur_feature(&self) -> &FgbFeature {
            &self.fbs
        }
        /// Read and process all selected features
        pub fn process_features<W: FeatureProcessor>(
            &mut self,
            out: &mut W,
        ) -> geozero::error::Result<()> {
            out.dataset_begin(self.fbs.header().name())?;
            let mut cnt = 0;
            while let Some(feature) = self
                .next()
                .map_err(|e| GeozeroError::Feature(e.to_string()))?
            {
                feature.process(out, cnt)?;
                cnt += 1;
            }
            out.dataset_end()
        }
    }

    impl<R: Read + Seek> FeatureIter<R, Seekable> {
        /// Return current feature
        pub fn cur_feature(&self) -> &FgbFeature {
            &self.fbs
        }
        /// Read and process all selected features
        pub fn process_features<W: FeatureProcessor>(
            &mut self,
            out: &mut W,
        ) -> geozero::error::Result<()> {
            out.dataset_begin(self.fbs.header().name())?;
            let mut cnt = 0;
            while let Some(feature) = self
                .next()
                .map_err(|e| GeozeroError::Feature(e.to_string()))?
            {
                feature.process(out, cnt)?;
                cnt += 1;
            }
            out.dataset_end()
        }
    }

    mod inspect {
        use super::*;
        use crate::packed_r_tree::PackedRTree;
        use crate::FgbReader;

        impl<R: Read> FgbReader<R> {
            /// Process R-Tree index for debugging purposes
            #[doc(hidden)]
            pub fn process_index<P: FeatureProcessor>(
                &mut self,
                processor: &mut P,
            ) -> geozero::error::Result<()> {
                let features_count = self.header().features_count() as usize;
                let index_node_size = self.header().index_node_size();
                let index =
                    PackedRTree::from_buf(&mut self.reader, features_count, index_node_size)
                        .map_err(|_| GeozeroError::GeometryIndex)?;
                index.process_index(processor)
            }
        }

        #[test]
        fn dump_index() -> geozero::error::Result<()> {
            use geozero::geojson::GeoJsonWriter;
            use std::fs::File;
            use std::io::{BufReader, BufWriter};

            let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
            let mut fgb = FgbReader::open(&mut filein).map_err(|_| GeozeroError::GeometryIndex)?;
            let mut fout = BufWriter::new(File::create("/tmp/countries-index.json")?);

            fgb.process_index(&mut GeoJsonWriter::new(&mut fout))?;
            Ok(())
        }
    }
}

// Shared FallibleStreamingIterator implementation
impl<R: Read, S> FeatureIter<R, S> {
    fn new(
        reader: R,
        verify: bool,
        fbs: FgbFeature,
        item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    ) -> FeatureIter<R, S> {
        let mut iter = FeatureIter {
            reader,
            verify,
            fbs,
            item_filter,
            count: None,
            feat_no: 0,
            cur_pos: 0,
            state: State::Init,
            seekable_marker: PhantomData,
        };

        if iter.read_feature_size() {
            iter.state = State::Finished
        } else {
            iter.state = State::ReadFirstFeatureSize
        };

        iter.count = match &iter.item_filter {
            Some(list) => Some(list.len()),
            None => {
                let feat_count = iter.fbs.header().features_count() as usize;
                if feat_count > 0 {
                    Some(feat_count)
                } else if iter.state == State::Finished {
                    Some(0)
                } else {
                    None
                }
            }
        };

        iter
    }

    /// Header information
    pub fn header(&self) -> Header {
        self.fbs.header()
    }

    /// Number of selected features (None for undefined feature count)
    pub fn features_count(&self) -> Option<usize> {
        self.count
    }

    fn advance_finished(&mut self) -> bool {
        if self.state == State::Finished {
            return true;
        }
        if let Some(count) = self.count {
            if self.feat_no >= count {
                self.state = State::Finished;
                return true;
            }
        }
        false
    }

    /// Read feature size and return true if end of dataset reached
    fn read_feature_size(&mut self) -> bool {
        self.fbs.feature_buf.resize(4, 0);
        self.cur_pos += 4;
        self.reader.read_exact(&mut self.fbs.feature_buf).is_err()
    }

    fn read_feature(&mut self) -> Result<()> {
        match self.state {
            State::ReadFirstFeatureSize => {
                self.state = State::Reading;
            }
            State::Reading => {
                if self.read_feature_size() {
                    self.state = State::Finished;
                    return Ok(());
                }
            }
            State::Finished => {
                debug_assert!(
                    false,
                    "shouldn't call read_feature on already finished Iter"
                );
                return Ok(());
            }
            State::Init => {
                unreachable!("should have read first feature size before reading any features")
            }
        }
        let sbuf = &self.fbs.feature_buf;
        let feature_size = u32::from_le_bytes([sbuf[0], sbuf[1], sbuf[2], sbuf[3]]) as usize;
        self.fbs.feature_buf.resize(feature_size + 4, 0);
        self.reader.read_exact(&mut self.fbs.feature_buf[4..])?;
        if self.verify {
            let _feature = size_prefixed_root_as_feature(&self.fbs.feature_buf)?;
        }
        self.feat_no += 1;
        self.cur_pos += feature_size as u64;
        Ok(())
    }

    fn iter_get(&self) -> Option<&FgbFeature> {
        if self.state == State::Finished {
            None
        } else {
            debug_assert!(self.state == State::Reading);
            Some(&self.fbs)
        }
    }

    fn iter_size_hint(&self) -> (usize, Option<usize>) {
        if self.state == State::Finished {
            (0, Some(0))
        } else if let Some(count) = self.count {
            let remaining = count - self.feat_no;
            (remaining, Some(remaining))
        } else {
            (0, None)
        }
    }
}

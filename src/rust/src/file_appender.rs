use crate::error::Result;
use crate::feature_writer::FeatureWriter;
use crate::header_generated::*;
use crate::packed_r_tree::{calc_extent_from_prev, NodeItem, PackedRTree};
use crate::properties_reader::FgbFeature;
use crate::Error;
use crate::{check_magic_bytes, HEADER_MAX_BUFFER_SIZE};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use flatbuffers::FlatBufferBuilder;
use geozero::CoordDimensions;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

/// FlatGeobuf dataset appender for mutable files
pub struct FgbAppender<'a, R> {
    reader: R,
    /// FlatBuffers verification
    verify: bool,
    // Original file data
    fbs: FgbFeature,
    // Writer components for new features
    tmpout: BufWriter<File>,
    fbb: FlatBufferBuilder<'a>,
    header_args: HeaderArgs<'a>,
    columns: Vec<flatbuffers::WIPOffset<Column<'a>>>,
    feat_writer: FeatureWriter<'a>,
    feat_offsets: Vec<FeatureOffset>,
    feat_nodes: Vec<NodeItem>,
}

#[derive(Debug)]
// Offsets in temporary file for new features
struct FeatureOffset {
    offset: usize,
    size: usize,
}

impl<'a, R: Read> FgbAppender<'a, R> {
    /// Open dataset by reading the header information
    pub fn open(reader: R) -> Result<FgbAppender<'a, R>> {
        Self::read_header(reader, true)
    }

    /// Open dataset by reading the header information without FlatBuffers verification
    ///
    /// # Safety
    /// This method is unsafe because it does not verify the FlatBuffers header.
    /// It is still safe from the Rust safety guarantees perspective, but it may cause
    /// undefined behavior if the FlatBuffers header is invalid.
    pub unsafe fn open_unchecked(reader: R) -> Result<FgbAppender<'a, R>> {
        Self::read_header(reader, false)
    }

    fn read_header(mut reader: R, verify: bool) -> Result<FgbAppender<'a, R>> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if !check_magic_bytes(&magic_buf) {
            return Err(Error::MissingMagicBytes);
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf) as usize;
        if header_size > HEADER_MAX_BUFFER_SIZE || header_size < 8 {
            return Err(Error::IllegalHeaderSize(header_size));
        }
        let mut header_buf = Vec::with_capacity(header_size + 4);
        header_buf.extend_from_slice(&size_buf);
        header_buf.resize(header_buf.capacity(), 0);
        reader.read_exact(&mut header_buf[4..])?;

        if verify {
            let _header = size_prefixed_root_as_header(&header_buf)?;
        }

        let fbs = FgbFeature {
            header_buf,
            feature_buf: Vec::new(),
        };
        let header = fbs.header();

        // Check if file is mutable
        if header.mutablity_version() == 0 {
            return Err(Error::Immutable);
        }

        // Initialize writer components
        let mut fbb = FlatBufferBuilder::new();

        // Copy header information for new writer
        let header_args = HeaderArgs {
            name: Some(fbb.create_string(header.name().unwrap_or(""))),
            geometry_type: header.geometry_type(),
            index_node_size: header.index_node_size(),
            mutability_version: header.mutablity_version(),
            crs: header.crs().map(|crs| {
                use crate::header_generated::{Crs, CrsArgs};
                let crs_args = CrsArgs {
                    org: crs.org().map(|v| fbb.create_string(v)),
                    code: crs.code(),
                    name: crs.name().map(|v| fbb.create_string(v)),
                    description: crs.description().map(|v| fbb.create_string(v)),
                    wkt: crs.wkt().map(|v| fbb.create_string(v)),
                    code_string: crs.code_string().map(|v| fbb.create_string(v)),
                };
                Crs::create(&mut fbb, &crs_args)
            }),
            has_z: header.has_z(),
            has_m: header.has_m(),
            has_t: header.has_t(),
            has_tm: header.has_tm(),
            title: header.title().map(|v| fbb.create_string(v)),
            description: header.description().map(|v| fbb.create_string(v)),
            metadata: header.metadata().map(|v| fbb.create_string(v)),
            features_count: header.features_count(),
            ..Default::default()
        };

        // Copy column information
        let mut columns = Vec::new();
        if let Some(cols) = header.columns() {
            for col in cols {
                let name = col.name();
                let title_offset = col.title().map(|v| fbb.create_string(v));
                let name_offset = Some(fbb.create_string(name));
                let description_offset = col.description().map(|v| fbb.create_string(v));
                let metadata_offset = col.metadata().map(|v| fbb.create_string(v));

                let col_args = ColumnArgs {
                    name: name_offset,
                    type_: col.type_(),
                    title: title_offset,
                    description: description_offset,
                    width: col.width(),
                    precision: col.precision(),
                    scale: col.scale(),
                    nullable: col.nullable(),
                    unique: col.unique(),
                    primary_key: col.primary_key(),
                    metadata: metadata_offset,
                };
                columns.push(Column::create(&mut fbb, &col_args));
            }
        }

        let dims = CoordDimensions {
            z: header_args.has_z,
            m: header_args.has_m,
            t: header_args.has_t,
            tm: header_args.has_tm,
        };
        let feat_writer = FeatureWriter::with_dims(
            header_args.geometry_type,
            true, // detect_type
            true, // promote_to_multi
            dims,
        );

        let tmpout = BufWriter::new(tempfile::tempfile()?);
        Ok(FgbAppender {
            reader,
            verify,
            fbs,
            tmpout,
            fbb,
            header_args,
            columns,
            feat_writer,
            feat_offsets: Vec::new(),
            feat_nodes: Vec::new(),
        })
    }

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

    /// Add a new column (similar to FgbWriter)
    pub fn add_column<F>(
        &mut self,
        name: &str,
        col_type: crate::header_generated::ColumnType,
        cfgfn: F,
    ) where
        F: FnOnce(&mut FlatBufferBuilder<'a>, &mut ColumnArgs),
    {
        let mut col = ColumnArgs {
            name: Some(self.fbb.create_string(name)),
            type_: col_type,
            ..Default::default()
        };
        cfgfn(&mut self.fbb, &mut col);
        self.columns.push(Column::create(&mut self.fbb, &col));
    }

    fn write_feature(&mut self) -> Result<()> {
        let mut node = self.feat_writer.bbox.clone();
        // Offset is index of feat_offsets before sorting
        node.offset = self.feat_offsets.len() as u64;
        self.feat_nodes.push(node);
        let feat_buf = self.feat_writer.finish_to_feature();
        let tmpoffset = self
            .feat_offsets
            .last()
            .map(|it| it.offset + it.size)
            .unwrap_or(0);
        self.feat_offsets.push(FeatureOffset {
            offset: tmpoffset,
            size: feat_buf.len(),
        });
        self.tmpout.write_all(&feat_buf)?;
        self.header_args.features_count += 1;
        Ok(())
    }
}

impl<'a, R: Read + Seek + Write> FgbAppender<'a, R> {
    /// Reindex and append new features to the file
    pub fn reindex_append(mut self, mut out: impl Write + Seek + Read) -> Result<()> {
        let header = self.fbs.header(); // current writter header
        if header.mutablity_version() == 0 {
            return Err(Error::Immutable);
        }

        // println!("out {}", out.stream_position()?);
        // reader should have read the header, while out stays at the beginning of the file at the starting
        // Load existing feature nodes for reindexing
        if header.index_node_size() > 0 && header.features_count() > 0 {
            let index_size = self.index_size();
            self.reader.seek(SeekFrom::End(-(index_size as i64)))?;
            let current_pos = self.reader.stream_position()?;
            let mut index_nodes = PackedRTree::nodes_from_buf(
                &mut self.reader,
                header.features_count() as usize,
                header.index_node_size(),
            )?;
            let prev_extent = header.envelope().unwrap();
            let extent = calc_extent_from_prev(
                &self.feat_nodes,
                &NodeItem {
                    min_x: prev_extent.get(0),
                    min_y: prev_extent.get(1),
                    max_x: prev_extent.get(2),
                    max_y: prev_extent.get(3),
                    offset: 0,
                },
            );
            // self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
            self.header_args.envelope = Some(self.fbb.create_vector(&[
                extent.min_x,
                extent.min_y,
                extent.max_x,
                extent.max_y,
            ]));
            self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
            let header = Header::create(&mut self.fbb, &self.header_args);
            self.fbb.finish_size_prefixed(header, None);
            let buf = self.fbb.finished_data();

            //write header and index must stay the same
            // println!("current pos: {}", out.stream_position()?);
            out.write_all(&MAGIC_BYTES)?;
            out.write_all(buf)?;
            // let last_offset = index_nodes.last().unwrap().offset;
            let header_end = out.stream_position()?;

            // println!("index nodes: {:?}", index_nodes);
            // println!("header end: {}, current pos: {}, last offset: {}", header_end, current_pos, last_offset);
            let new_offset_start = current_pos - header_end;

            // let last_offset = index_nodes.last().map(|it| it.offset).unwrap_or(0);
            let feat_nodes = self
                .feat_nodes
                .iter()
                .map(|it| {
                    let mut it = it.clone();
                    it.offset += new_offset_start;
                    it
                })
                .collect::<Vec<_>>();

            index_nodes.extend(feat_nodes);
            let index =
                PackedRTree::build(&index_nodes, &extent, self.header_args.index_node_size)?;
            let _ = out.seek(SeekFrom::Start(current_pos));

            //output the new nodes
            self.tmpout.rewind()?;
            let unsorted_feature_output = self.tmpout.into_inner().map_err(|e| e.into_error())?;
            let mut unsorted_feature_reader = BufReader::new(unsorted_feature_output);
            std::io::copy(&mut unsorted_feature_reader, &mut out)?;

            //output the index
            index.stream_write(&mut out)?;
        }

        Ok(())
    }
}

// Implement geozero API similar to FgbWriter
mod geozero_api {
    use super::*;
    use crate::feature_writer::{prop_type, FeatureWriter};
    use geozero::error::GeozeroError;
    use geozero::{
        error::Result, ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource,
        GeozeroGeometry, PropertyProcessor,
    };

    impl<R: Read> FgbAppender<'_, R> {
        /// Add a new feature
        pub fn add_feature(&mut self, mut feature: impl GeozeroDatasource) -> Result<()> {
            feature.process(&mut self.feat_writer)?;
            self.write_feature()
                .map_err(|e| GeozeroError::Feature(e.to_string()))
        }

        /// Add a new feature from a `GeozeroGeometry`
        pub fn add_feature_geom<F>(&mut self, geom: impl GeozeroGeometry, cfgfn: F) -> Result<()>
        where
            F: FnOnce(&mut FeatureWriter),
        {
            geom.process_geom(&mut self.feat_writer)?;
            cfgfn(&mut self.feat_writer);
            self.write_feature()
                .map_err(|e| GeozeroError::Feature(e.to_string()))
        }
    }

    impl<R: Read> FeatureProcessor for FgbAppender<'_, R> {
        fn feature_end(&mut self, _idx: u64) -> Result<()> {
            self.write_feature()
                .map_err(|e| GeozeroError::Feature(e.to_string()))
        }
    }

    impl<R: Read> PropertyProcessor for FgbAppender<'_, R> {
        fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
            if i >= self.columns.len() {
                if i == self.columns.len() {
                    println!(
                        "Undefined property index {i}, column: `{colname}` - adding column declaration"
                    );
                    self.add_column(colname, prop_type(colval), |_, _| {});
                } else {
                    println!("Undefined property index {i}, column: `{colname}` - skipping");
                    return Ok(false);
                }
            }
            self.feat_writer.property(i, colname, colval)
        }
    }

    // Delegate GeomProcessor to feat_writer
    impl<R: Read> GeomProcessor for FgbAppender<'_, R> {
        fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
            self.feat_writer.xy(x, y, idx)
        }
        fn coordinate(
            &mut self,
            x: f64,
            y: f64,
            z: Option<f64>,
            m: Option<f64>,
            t: Option<f64>,
            tm: Option<u64>,
            idx: usize,
        ) -> Result<()> {
            self.feat_writer.coordinate(x, y, z, m, t, tm, idx)
        }
        fn point_begin(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.point_begin(idx)
        }
        fn point_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.point_end(idx)
        }
        fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.multipoint_begin(size, idx)
        }
        fn multipoint_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.multipoint_end(idx)
        }
        fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.linestring_begin(tagged, size, idx)
        }
        fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
            self.feat_writer.linestring_end(tagged, idx)
        }
        fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.multilinestring_begin(size, idx)
        }
        fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.multilinestring_end(idx)
        }
        fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.polygon_begin(tagged, size, idx)
        }
        fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
            self.feat_writer.polygon_end(tagged, idx)
        }
        fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.multipolygon_begin(size, idx)
        }
        fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.multipolygon_end(idx)
        }
        fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.circularstring_begin(size, idx)
        }
        fn circularstring_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.circularstring_end(idx)
        }
        fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.compoundcurve_begin(size, idx)
        }
        fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.compoundcurve_end(idx)
        }
        fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.curvepolygon_begin(size, idx)
        }
        fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.curvepolygon_end(idx)
        }
        fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.multicurve_begin(size, idx)
        }
        fn multicurve_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.multicurve_end(idx)
        }
        fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.multisurface_begin(size, idx)
        }
        fn multisurface_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.multisurface_end(idx)
        }
        fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.triangle_begin(tagged, size, idx)
        }
        fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
            self.feat_writer.triangle_end(tagged, idx)
        }
        fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.polyhedralsurface_begin(size, idx)
        }
        fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.polyhedralsurface_end(idx)
        }
        fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
            self.feat_writer.tin_begin(size, idx)
        }
        fn tin_end(&mut self, idx: usize) -> Result<()> {
            self.feat_writer.tin_end(idx)
        }
    }
}

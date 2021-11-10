use crate::feature_writer::FeatureWriter;
use crate::header_generated::{ColumnType, Crs, CrsArgs, GeometryType};
use crate::packed_r_tree::{calc_extent, hilbert_sort, NodeItem, PackedRTree};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use geozero::error::Result;
use geozero::{CoordDimensions, GeozeroDatasource, GeozeroGeometry};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// FlatGeobuf dataset writer
pub struct FgbWriter<'a> {
    tmpfn: PathBuf,
    tmpout: BufWriter<NamedTempFile>,
    fbb: flatbuffers::FlatBufferBuilder<'a>,
    header_args: HeaderArgs<'a>,
    columns: Vec<flatbuffers::WIPOffset<Column<'a>>>,
    feat_writer: FeatureWriter<'a>,
    feat_offsets: Vec<FeatureOffset>,
    feat_nodes: Vec<NodeItem>,
}

// Offsets in temporary file
struct FeatureOffset {
    offset: usize,
    size: usize,
}

impl<'a> FgbWriter<'a> {
    /// Configure FlatGeobuf headers for creating a new file
    ///
    /// * For reading/writing more than two dimensions set `hasZ=true`, etc.
    /// * For skipping the index, set `index_node_size=0`
    ///
    /// # Usage example:
    ///
    /// ```
    /// # use flatgeobuf::*;
    /// let mut fgb = FgbWriter::create(
    ///     "countries",
    ///     GeometryType::MultiPolygon,
    ///     Some(4326),
    ///     |header| {
    ///         header.description = Some(FgbWriter::create_string("Country polygons"));
    ///     },
    /// ).unwrap();
    /// ```
    pub fn create<F>(
        name: &str,
        geometry_type: GeometryType,
        crs_code: Option<i32>,
        cfgfn: F,
    ) -> Result<Self>
    where
        F: FnOnce(&mut HeaderArgs),
    {
        let mut fbb = flatbuffers::FlatBufferBuilder::new();

        let crs = crs_code.map(|code| {
            Crs::create(
                &mut fbb,
                &CrsArgs {
                    code,
                    ..Default::default()
                },
            )
        });

        let mut header_args = HeaderArgs {
            name: Some(fbb.create_string(name)),
            geometry_type,
            crs,
            index_node_size: PackedRTree::DEFAULT_NODE_SIZE,
            ..Default::default()
        };

        cfgfn(&mut header_args);

        let mut feat_writer = FeatureWriter::new();
        feat_writer.dims = CoordDimensions {
            z: header_args.hasZ,
            m: header_args.hasM,
            t: header_args.hasT,
            tm: header_args.hasTM,
        };

        let tmpfile = NamedTempFile::new()?;
        let tmpfn = tmpfile.path().to_path_buf();
        let tmpout = BufWriter::new(tmpfile);

        Ok(FgbWriter {
            tmpfn,
            tmpout,
            fbb,
            header_args,
            columns: Vec::new(),
            feat_writer,
            feat_offsets: Vec::new(),
            feat_nodes: Vec::new(),
        })
    }

    /// Create a builder for FlatBuffer entities.
    pub fn fb_builder() -> flatbuffers::FlatBufferBuilder<'a> {
        flatbuffers::FlatBufferBuilder::new()
    }

    /// Create a single FlatBuffer string.
    pub fn create_string(val: &'a str) -> flatbuffers::WIPOffset<&str> {
        Self::fb_builder().create_string(val)
    }

    /// Add a new column.
    pub fn add_column<F>(&mut self, name: &str, col_type: ColumnType, cfgfn: F)
    where
        F: FnOnce(&mut ColumnArgs),
    {
        let mut col = ColumnArgs {
            name: Some(self.fbb.create_string(name)),
            type_: col_type,
            ..Default::default()
        };
        cfgfn(&mut col);
        self.columns.push(Column::create(&mut self.fbb, &col));
    }

    /// Add a new feature.
    pub fn add_feature(&mut self, mut feature: impl GeozeroDatasource) -> Result<()> {
        feature.process(&mut self.feat_writer)?;
        self.write_feature()
    }

    /// Add a new feature from a `GeozeroGeometry`.
    pub fn add_feature_geom<F>(&mut self, geom: impl GeozeroGeometry, cfgfn: F) -> Result<()>
    where
        F: FnOnce(&mut FeatureWriter),
    {
        geom.process_geom(&mut self.feat_writer)?;
        cfgfn(&mut self.feat_writer);
        self.write_feature()
    }

    fn write_feature(&mut self) -> Result<()> {
        let mut node = self.feat_writer.bbox.clone();
        // Offset is index of feat_offsets before sorting
        // Will be replaced with output offset after sorting
        node.offset = self.feat_offsets.len();
        self.feat_nodes.push(node);
        let feat_buf = self.feat_writer.to_feature();
        let tmpoffset = self
            .feat_offsets
            .last()
            .map(|it| it.offset + it.size)
            .unwrap_or(0);
        self.feat_offsets.push(FeatureOffset {
            offset: tmpoffset,
            size: feat_buf.len(),
        });
        self.tmpout.write(&feat_buf)?;
        self.header_args.features_count += 1;
        Ok(())
    }

    /// Write the FlatGeobuf dataset (Hilbert sorted)
    pub fn write<W: Write>(&mut self, out: &'a mut W) -> Result<()> {
        out.write(&MAGIC_BYTES)?;

        let extent = calc_extent(&self.feat_nodes);

        // Write header
        self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
        self.header_args.envelope =
            Some(
                self.fbb
                    .create_vector(&[extent.min_x, extent.min_y, extent.max_x, extent.max_y]),
            );
        let header = Header::create(&mut self.fbb, &self.header_args);
        self.fbb.finish_size_prefixed(header, None);
        let buf = self.fbb.finished_data();
        out.write(&buf)?;

        if self.header_args.index_node_size > 0 {
            // Create sorted index
            hilbert_sort(&mut self.feat_nodes, &extent);
            // Update offsets for index
            let mut offset = 0;
            let index_nodes = self
                .feat_nodes
                .iter()
                .map(|tmpnode| {
                    let feat = &self.feat_offsets[tmpnode.offset];
                    let mut node = tmpnode.clone();
                    node.offset = offset;
                    offset += feat.size;
                    node
                })
                .collect();
            let tree = PackedRTree::build(&index_nodes, &extent, self.header_args.index_node_size)?;
            tree.stream_write(out)?;
        }

        // Copy features from temp file in sort order
        self.tmpout.flush()?;
        let tmpin = File::open(&self.tmpfn)?;
        let mut reader = BufReader::new(tmpin);
        let mut buf = Vec::with_capacity(2048);
        for node in &self.feat_nodes {
            let feat = &self.feat_offsets[node.offset];
            reader.seek(SeekFrom::Start(feat.offset as u64))?;
            buf.resize(feat.size, 0);
            reader.read_exact(&mut buf)?;
            out.write(&buf)?;
        }

        Ok(())
    }
}

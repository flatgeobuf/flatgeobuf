use crate::feature_writer::{prop_type, FeatureWriter};
use crate::header_generated::{ColumnType, Crs, CrsArgs, GeometryType};
use crate::packed_r_tree::{calc_extent, hilbert_sort, NodeItem, PackedRTree};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use geozero::error::Result;
use geozero::{
    ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, GeozeroDatasource,
    GeozeroGeometry, PropertyProcessor,
};
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
        node.offset = self.feat_offsets.len() as u64;
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

        if self.header_args.index_node_size > 0 && self.feat_nodes.len() > 0 {
            // Create sorted index
            hilbert_sort(&mut self.feat_nodes, &extent);
            // Update offsets for index
            let mut offset = 0;
            let index_nodes = self
                .feat_nodes
                .iter()
                .map(|tmpnode| {
                    let feat = &self.feat_offsets[tmpnode.offset as usize];
                    let mut node = tmpnode.clone();
                    node.offset = offset;
                    offset += feat.size as u64;
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
            let feat = &self.feat_offsets[node.offset as usize];
            reader.seek(SeekFrom::Start(feat.offset as u64))?;
            buf.resize(feat.size, 0);
            reader.read_exact(&mut buf)?;
            out.write(&buf)?;
        }

        Ok(())
    }
}

impl FeatureProcessor for FgbWriter<'_> {
    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        self.write_feature()
    }
}

impl PropertyProcessor for FgbWriter<'_> {
    fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
        if i >= self.columns.len() {
            if i == self.columns.len() {
                info!(
                    "Undefined property index {}, column: `{}` - adding column declaration",
                    i, colname
                );
                self.add_column(colname, prop_type(colval), |_| {});
            } else {
                info!(
                    "Undefined property index {}, column: `{}` - skipping",
                    i, colname
                );
                return Ok(false);
            }
        }
        // TODO: check name and type against existing declartion
        self.feat_writer.property(i, colname, colval)
    }
}

// Delegate GeomProcessor to self.feat_writer
impl GeomProcessor for FgbWriter<'_> {
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
        self.header_args.geometry_type = GeometryType::Point;
        self.feat_writer.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::MultiPoint;
        self.feat_writer.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        if tagged {
            self.header_args.geometry_type = GeometryType::LineString;
        }
        self.feat_writer.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.feat_writer.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::MultiLineString;
        self.feat_writer.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        if tagged {
            self.header_args.geometry_type = GeometryType::Polygon;
        }
        self.feat_writer.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.feat_writer.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::MultiPolygon;
        self.feat_writer.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.multipolygon_end(idx)
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::CircularString;
        self.feat_writer.circularstring_begin(size, idx)
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.circularstring_end(idx)
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::CompoundCurve;
        self.feat_writer.compoundcurve_begin(size, idx)
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.compoundcurve_end(idx)
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::CurvePolygon;
        self.feat_writer.curvepolygon_begin(size, idx)
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.curvepolygon_end(idx)
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::MultiCurve;
        self.feat_writer.multicurve_begin(size, idx)
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.multicurve_end(idx)
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::MultiSurface;
        self.feat_writer.multisurface_begin(size, idx)
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.multisurface_end(idx)
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        if tagged {
            self.header_args.geometry_type = GeometryType::Triangle;
        }
        self.feat_writer.triangle_begin(tagged, size, idx)
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.feat_writer.triangle_end(tagged, idx)
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::PolyhedralSurface;
        self.feat_writer.polyhedralsurface_begin(size, idx)
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.polyhedralsurface_end(idx)
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.header_args.geometry_type = GeometryType::TIN;
        self.feat_writer.tin_begin(size, idx)
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.feat_writer.tin_end(idx)
    }
}

use crate::error::Result;
use crate::feature_writer::FeatureWriter;
use crate::header_generated::{ColumnType, Crs, CrsArgs, GeometryType};
use crate::packed_r_tree::{calc_extent, hilbert_sort, NodeItem, PackedRTree};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use flatbuffers::FlatBufferBuilder;
use geozero::CoordDimensions;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

/// FlatGeobuf dataset writer
///
/// # Usage example:
///
/// ```
/// use flatgeobuf::*;
/// use geozero::geojson::GeoJsonReader;
/// use geozero::GeozeroDatasource;
/// # use std::fs::File;
/// # use std::io::{BufReader, BufWriter};
///
/// # fn json_to_fgb() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// let mut fgb = FgbWriter::create("countries", GeometryType::MultiPolygon)?;
/// let mut fin = BufReader::new(File::open("countries.geojson")?);
/// let mut reader = GeoJsonReader(&mut fin);
/// reader.process(&mut fgb)?;
/// let mut fout = BufWriter::new(File::create("countries.fgb")?);
/// fgb.write(&mut fout)?;
/// # Ok(())
/// # }
/// ```
pub struct FgbWriter<'a> {
    tmpout: BufWriter<File>,
    fbb: FlatBufferBuilder<'a>,
    header_args: HeaderArgs<'a>,
    columns: Vec<flatbuffers::WIPOffset<Column<'a>>>,
    feat_writer: FeatureWriter<'a>,
    feat_offsets: Vec<FeatureOffset>,
    feat_nodes: Vec<NodeItem>,
}

/// Options for FlatGeobuf writer
#[derive(Debug)]
pub struct FgbWriterOptions<'a> {
    /// Write index and sort features accordingly.
    pub write_index: bool,
    /// Detect geometry type when `geometry_type` is Unknown.
    pub detect_type: bool,
    /// Convert single to multi geometries, if `geometry_type` is multi type or Unknown
    pub promote_to_multi: bool,
    /// CRS definition
    pub crs: FgbCrs<'a>,
    /// Does geometry have Z dimension?
    pub has_z: bool,
    /// Does geometry have M dimension?
    pub has_m: bool,
    /// Does geometry have T dimension?
    pub has_t: bool,
    /// Does geometry have TM dimension?
    pub has_tm: bool,
    // Dataset title
    pub title: Option<&'a str>,
    // Dataset description (intended for free form long text)
    pub description: Option<&'a str>,
    // Dataset metadata (intended to be application specific and
    pub metadata: Option<&'a str>,
}

impl Default for FgbWriterOptions<'_> {
    fn default() -> Self {
        FgbWriterOptions {
            write_index: true,
            detect_type: true,
            promote_to_multi: true,
            crs: Default::default(),
            has_z: false,
            has_m: false,
            has_t: false,
            has_tm: false,
            title: None,
            description: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct FgbCrs<'a> {
    /// Case-insensitive name of the defining organization e.g. EPSG or epsg (NULL = EPSG)
    pub org: Option<&'a str>,
    /// Numeric ID of the Spatial Reference System assigned by the organization (0 = unknown)
    pub code: i32,
    /// Human readable name of this SRS
    pub name: Option<&'a str>,
    /// Human readable description of this SRS
    pub description: Option<&'a str>,
    /// Well-known Text Representation of the Spatial Reference System
    pub wkt: Option<&'a str>,
    /// Text ID of the Spatial Reference System assigned by the organization in the (rare) case when it is not an integer and thus cannot be set into code
    pub code_string: Option<&'a str>,
}

#[derive(Debug)]
// Offsets in temporary file
struct FeatureOffset {
    offset: usize,
    size: usize,
}

impl<'a> FgbWriter<'a> {
    /// Configure FlatGeobuf headers for creating a new file with default options
    ///
    /// # Usage example:
    ///
    /// ```
    /// # use flatgeobuf::*;
    /// let mut fgb = FgbWriter::create("countries", GeometryType::MultiPolygon).unwrap();
    /// ```
    pub fn create(name: &str, geometry_type: GeometryType) -> Result<Self> {
        let options = FgbWriterOptions {
            write_index: true,
            detect_type: true,
            promote_to_multi: true,
            ..Default::default()
        };
        FgbWriter::create_with_options(name, geometry_type, options)
    }
    /// Configure FlatGeobuf headers for creating a new file
    ///
    /// # Usage example:
    ///
    /// ```
    /// # use flatgeobuf::*;
    /// let mut fgb = FgbWriter::create_with_options(
    ///     "countries",
    ///     GeometryType::MultiPolygon,
    ///     FgbWriterOptions {
    ///         description: Some("Country polygons"),
    ///         write_index: false,
    ///         crs: FgbCrs {
    ///             code: 4326,
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     },
    /// )
    /// .unwrap();
    /// ```
    pub fn create_with_options(
        name: &str,
        geometry_type: GeometryType,
        options: FgbWriterOptions,
    ) -> Result<Self> {
        let mut fbb = FlatBufferBuilder::new();

        let index_node_size = if options.write_index {
            PackedRTree::DEFAULT_NODE_SIZE
        } else {
            0
        };
        let crs_args = CrsArgs {
            org: options.crs.org.map(|v| fbb.create_string(v)),
            code: options.crs.code,
            name: options.crs.name.map(|v| fbb.create_string(v)),
            description: options.crs.description.map(|v| fbb.create_string(v)),
            wkt: options.crs.wkt.map(|v| fbb.create_string(v)),
            code_string: options.crs.code_string.map(|v| fbb.create_string(v)),
        };
        let header_args = HeaderArgs {
            name: Some(fbb.create_string(name)),
            geometry_type,
            index_node_size,
            crs: Some(Crs::create(&mut fbb, &crs_args)),
            has_z: options.has_z,
            has_m: options.has_m,
            has_t: options.has_t,
            has_tm: options.has_tm,
            title: options.title.map(|v| fbb.create_string(v)),
            description: options.description.map(|v| fbb.create_string(v)),
            metadata: options.metadata.map(|v| fbb.create_string(v)),
            ..Default::default()
        };

        let dims = CoordDimensions {
            z: header_args.has_z,
            m: header_args.has_m,
            t: header_args.has_t,
            tm: header_args.has_tm,
        };
        let feat_writer = FeatureWriter::with_dims(
            header_args.geometry_type,
            options.detect_type,
            options.promote_to_multi,
            dims,
        );

        let tmpout = BufWriter::new(tempfile::tempfile()?);

        Ok(FgbWriter {
            tmpout,
            fbb,
            header_args,
            columns: Vec::new(),
            feat_writer,
            feat_offsets: Vec::new(),
            feat_nodes: Vec::new(),
        })
    }

    /// Add a new column.
    ///
    /// # Usage example:
    ///
    /// ```
    /// # use flatgeobuf::*;
    /// # let mut fgb = FgbWriter::create("", GeometryType::Point).unwrap();
    /// fgb.add_column("fid", ColumnType::ULong, |_fbb, col| {
    ///     col.nullable = false;
    /// });
    /// ```
    pub fn add_column<F>(&mut self, name: &str, col_type: ColumnType, cfgfn: F)
    where
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
        // Will be replaced with output offset after sorting
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

    /// Write the FlatGeobuf dataset (Hilbert sorted)
    pub fn write(mut self, mut out: impl Write) -> Result<()> {
        out.write_all(&MAGIC_BYTES)?;

        let extent = calc_extent(&self.feat_nodes);

        // Write header
        self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
        self.header_args.envelope =
            Some(
                self.fbb
                    .create_vector(&[extent.min_x, extent.min_y, extent.max_x, extent.max_y]),
            );
        self.header_args.geometry_type = self.feat_writer.dataset_type;
        let header = Header::create(&mut self.fbb, &self.header_args);
        self.fbb.finish_size_prefixed(header, None);
        let buf = self.fbb.finished_data();
        out.write_all(buf)?;

        if self.header_args.index_node_size > 0 && !self.feat_nodes.is_empty() {
            // Create sorted index
            println!("Hilbert sorting {} features", self.feat_nodes.len());
            hilbert_sort(&mut self.feat_nodes, &extent);
            // Update offsets for index
            // let mut offset = 0;
            let index_nodes = self
                .feat_nodes
                .iter()
                .map(|tmpnode| {
                    // let feat = &self.feat_offsets[tmpnode.offset as usize];
                    let mut node = tmpnode.clone();
                    node.offset = self.feat_offsets[tmpnode.offset as usize].offset as u64;
                    // node.offset = 1;
                    // offset += feat.size as u64;
                    node
                })
                .collect::<Vec<_>>();
            let tree = PackedRTree::build(&index_nodes, &extent, self.header_args.index_node_size)?;
            tree.stream_write(&mut out)?;
        }
        // println!("Writing {} features", self.feat_offsets.len());
        // Copy features from temp file in sort order
        self.tmpout.rewind()?;
        let unsorted_feature_output = self.tmpout.into_inner().map_err(|e| e.into_error())?;
        let mut unsorted_feature_reader = BufReader::new(unsorted_feature_output);
        std::io::copy(&mut unsorted_feature_reader, &mut out)?;
        // unsorted_feature_reader.seek(SeekFrom::Start(0));
        // buf.resize(0, 0);
        // out.write_all()?;
        // Clippy generates a false-positive here, needs a block to disable, see
        // https://github.com/rust-lang/rust-clippy/issues/9274
        // #[allow(clippy::read_zero_byte_vec)]
        // {
        //     let mut buf = Vec::with_capacity(2048);
            
        //     for node in 0..(&self.feat_nodes).len() {

        //         let feat = &self.feat_offsets[node];
        //         unsorted_feature_reader.seek(SeekFrom::Start(feat.offset as u64))?;
        //         buf.resize(feat.size, 0);
        //         unsorted_feature_reader.read_exact(&mut buf)?;
        //         out.write_all(&buf)?;
        //     }
        // }

        Ok(())
    }

    // pub fn reindex_write(&mut self, mut out: impl Write+Read, header : Header) -> Result<()> {
    //     // out.write_all(&MAGIC_BYTES)?;

    //     let extent = calc_extent(&self.feat_nodes);
    //     // No need to rewrite the whole header, do later
    //     // Write header
    //     self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
    //     self.header_args.envelope =
    //         Some(
    //             self.fbb
    //                 .create_vector(&[extent.min_x, extent.min_y, extent.max_x, extent.max_y]),
    //         );
    //     self.header_args.geometry_type = self.feat_writer.dataset_type;
    //     let header = Header::create(&mut self.fbb, &self.header_args);
    //     self.fbb.finish_size_prefixed(header, None);
    //     let buf = self.fbb.finished_data();
    //     out.write_all(buf)?;
        

    //     if self.header_args.index_node_size > 0 && !self.feat_nodes.is_empty() {
    //         // Create sorted index
    //         hilbert_sort(&mut self.feat_nodes, &extent);
    //         // Update offsets for index
    //         // let mut offset = 0;
    //         let index_nodes = self
    //             .feat_nodes
    //             .iter()
    //             .map(|tmpnode| {
    //                 // let feat = &self.feat_offsets[tmpnode.offset as usize];
    //                 let mut node = tmpnode.clone();
    //                 node.offset = self.feat_offsets[tmpnode.offset as usize].offset as u64;
    //                 // offset += feat.size as u64;
    //                 node
    //             })
    //             .collect::<Vec<_>>();
    //         let tree = PackedRTree::build(&index_nodes, &extent, self.header_args.index_node_size)?;
    //         tree.stream_write(&mut out)?;
    //     }

    //     // Copy features from temp file in sort order
    //     self.tmpout.rewind()?;
    //     let unsorted_feature_output = self.tmpout.into_inner().map_err(|e| e.into_error())?;
    //     let mut unsorted_feature_reader = BufReader::new(unsorted_feature_output);
        
    //     // Clippy generates a false-positive here, needs a block to disable, see
    //     // https://github.com/rust-lang/rust-clippy/issues/9274
    //     #[allow(clippy::read_zero_byte_vec)]
    //     {
    //         let mut buf = Vec::with_capacity(2048);
            
    //         for node in 0..(&self.feat_nodes).len() {

    //             let feat = &self.feat_offsets[node];
    //             unsorted_feature_reader.seek(SeekFrom::Start(feat.offset as u64))?;
    //             buf.resize(feat.size, 0);
    //             unsorted_feature_reader.read_exact(&mut buf)?;
    //             out.write_all(&buf)?;
    //         }
    //     }

    //     Ok(())
    // }
}

mod geozero_api {
    use crate::feature_writer::{prop_type, FeatureWriter};
    use crate::FgbWriter;
    use geozero::error::GeozeroError;
    use geozero::{
        error::Result, ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource,
        GeozeroGeometry, PropertyProcessor,
    };

    impl FgbWriter<'_> {
        /// Add a new feature.
        ///
        /// # Usage example:
        ///
        /// ```
        /// # use flatgeobuf::*;
        /// use geozero::geojson::GeoJson;
        /// # let mut fgb = FgbWriter::create("", GeometryType::Point).unwrap();
        /// let geojson = GeoJson(r#"{"type": "Feature", "properties": {"fid": 42, "name": "New Zealand"}, "geometry": {"type": "Point", "coordinates": [1, 1]}}"#);
        /// fgb.add_feature(geojson).ok();
        /// ```
        pub fn add_feature(&mut self, mut feature: impl GeozeroDatasource) -> Result<()> {
            feature.process(&mut self.feat_writer)?;
            self.write_feature()
                .map_err(|e| GeozeroError::Feature(e.to_string()))
        }

        /// Add a new feature from a `GeozeroGeometry`.
        ///
        /// # Usage example:
        ///
        /// ```
        /// # use flatgeobuf::*;
        /// use geozero::geojson::GeoJson;
        /// use geozero::{ColumnValue, PropertyProcessor};
        /// # let mut fgb = FgbWriter::create("", GeometryType::Point).unwrap();
        /// let geom = GeoJson(r#"{"type": "Point", "coordinates": [1, 1]}"#);
        /// fgb.add_feature_geom(geom, |feat| {
        ///     feat.property(0, "fid", &ColumnValue::Long(43)).unwrap();
        ///     feat.property(1, "name", &ColumnValue::String("South Africa"))
        ///         .unwrap();
        /// })
        /// .ok();
        /// ```
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

    impl FeatureProcessor for FgbWriter<'_> {
        fn feature_end(&mut self, _idx: u64) -> Result<()> {
            self.write_feature()
                .map_err(|e| GeozeroError::Feature(e.to_string()))
        }
    }

    impl PropertyProcessor for FgbWriter<'_> {
        fn property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
            if i >= self.columns.len() {
                if i == self.columns.len() {
                    info!(
                    "Undefined property index {i}, column: `{colname}` - adding column declaration"
                );
                    self.add_column(colname, prop_type(colval), |_, _| {});
                } else {
                    info!("Undefined property index {i}, column: `{colname}` - skipping");
                    return Ok(false);
                }
            }
            // TODO: check name and type against existing declaration
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

use crate::feature_writer::FeatureWriter;
use crate::header_generated::{ColumnType, Crs, CrsArgs, GeometryType};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use geozero::error::Result;
use geozero::{CoordDimensions, GeozeroDatasource, GeozeroGeometry};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// FlatGeobuf dataset writer
pub struct FgbWriter<'a> {
    tmpfn: PathBuf,
    tmpout: BufWriter<NamedTempFile>,
    fbb: flatbuffers::FlatBufferBuilder<'a>,
    pub header_args: HeaderArgs<'a>,
    columns: Vec<flatbuffers::WIPOffset<Column<'a>>>,
    feat_writer: FeatureWriter<'a>,
}

impl<'a> FgbWriter<'a> {
    pub fn create<F>(
        name: &str,
        geometry_type: GeometryType,
        crs_code: Option<i32>,
        cfgfn: F,
    ) -> Self
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
            index_node_size: 0,
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

        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfn = tmpfile.path().to_path_buf();
        let tmpout = BufWriter::new(tmpfile);

        FgbWriter {
            tmpfn,
            tmpout,
            fbb,
            header_args,
            columns: Vec::new(),
            feat_writer,
        }
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
        self.write_feature().unwrap();
        Ok(())
    }
    /// Add a new feature from a `GeozeroGeometry`.
    pub fn add_feature_geom<F>(&mut self, geom: impl GeozeroGeometry, cfgfn: F) -> Result<()>
    where
        F: FnOnce(&mut FeatureWriter),
    {
        geom.process_geom(&mut self.feat_writer)?;
        cfgfn(&mut self.feat_writer);
        self.write_feature().unwrap();
        Ok(())
    }
    fn write_feature(&mut self) -> std::io::Result<usize> {
        let feat_buf = self.feat_writer.to_feature();
        self.tmpout.write(&feat_buf)?;
        self.header_args.features_count += 1;
        Ok(0)
    }
    /// Write the FlatGeobuf dataset without index
    pub fn write_without_index<W: Write>(&mut self, out: &'a mut W) -> std::io::Result<()> {
        out.write(&MAGIC_BYTES)?;

        // Write header
        self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
        let header = Header::create(&mut self.fbb, &self.header_args);
        self.fbb.finish_size_prefixed(header, None);
        let buf = self.fbb.finished_data();
        out.write(&buf)?;

        // Copy features from temp file
        self.tmpout.flush()?;
        let tmpin = File::open(&self.tmpfn)?;
        let mut reader = BufReader::new(tmpin);
        std::io::copy(&mut reader, out)?;

        Ok(())
    }
    /// Write the Hilbert sorted FlatGeobuf dataset
    pub fn write<W: Write>(&mut self, out: &'a mut W) -> std::io::Result<()> {
        self.write_without_index(out) //TODO
    }
}

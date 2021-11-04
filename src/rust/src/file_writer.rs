use crate::feature_writer::FeatureWriter;
use crate::header_generated::{ColumnType, GeometryType};
use crate::{Column, ColumnArgs, Header, HeaderArgs, MAGIC_BYTES};
use geozero::error::Result;
use geozero::{ColumnValue, GeozeroDatasource, GeozeroGeometry, PropertyProcessor};
use std::io::Write;

/// FlatGeobuf dataset writer
pub struct FgbWriter<'a, W: Write> {
    writer: &'a mut W,
    fbb: flatbuffers::FlatBufferBuilder<'a>,
    pub header_args: HeaderArgs<'a>,
    columns: Vec<flatbuffers::WIPOffset<Column<'a>>>,
    feat_writer: FeatureWriter<'a>,
}

impl<'a, W: Write> FgbWriter<'a, W> {
    pub fn new(writer: &'a mut W, name: &str, geometry_type: GeometryType) -> Self {
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let header_args = HeaderArgs {
            name: Some(fbb.create_string(name)),
            geometry_type,
            index_node_size: 0,
            ..Default::default()
        };
        FgbWriter {
            writer,
            fbb,
            header_args,
            columns: Vec::new(),
            feat_writer: FeatureWriter::new(),
        }
    }
    pub fn write_magic(&mut self) -> std::io::Result<usize> {
        self.writer.write(&MAGIC_BYTES)
    }
    pub fn add_column(&mut self, name: &str, col_type: ColumnType) {
        let col = ColumnArgs {
            name: Some(self.fbb.create_string(name)),
            type_: col_type,
            ..Default::default()
        };
        self.columns.push(Column::create(&mut self.fbb, &col));
    }
    pub fn write_header(&mut self) -> std::io::Result<usize> {
        self.header_args.columns = Some(self.fbb.create_vector(&self.columns));
        let header = Header::create(&mut self.fbb, &self.header_args);
        self.fbb.finish_size_prefixed(header, None);
        let buf = self.fbb.finished_data();
        self.writer.write(&buf)
    }
    pub fn add_feature(&mut self, mut feature: impl GeozeroDatasource) -> Result<()> {
        self.header_args.features_count += 1;
        feature.process(&mut self.feat_writer)
    }
    pub fn add_feature_geom(&mut self, geom: impl GeozeroGeometry) -> Result<()> {
        self.header_args.features_count += 1;
        geom.process_geom(&mut self.feat_writer)
    }
    pub fn set_property(&mut self, i: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
        // TODO: check colval against columns_meta.get(i).type_() - requires header access
        self.feat_writer.property(i, colname, colval)
    }
    pub fn write_feature(&mut self) -> std::io::Result<usize> {
        let feat_buf = self.feat_writer.to_feature();
        self.writer.write(&feat_buf)
    }
}

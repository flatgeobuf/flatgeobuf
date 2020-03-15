use crate::feature_generated::flat_geobuf::{Feature, Geometry};
use crate::header_generated::flat_geobuf::GeometryType;
use crate::reader::{read_geometry, read_properties, ColumnMeta, ColumnValue, GeomReader};
use std::fmt::Display;
use std::io::Write;

struct GeoJsonEmitter<'a, W: Write> {
    out: &'a mut W,
}

impl<'a, W: Write> GeoJsonEmitter<'a, W> {
    fn new(out: &'a mut W) -> GeoJsonEmitter<'a, W> {
        GeoJsonEmitter { out }
    }
    fn comma(&mut self, idx: usize) {
        if idx > 0 {
            self.out.write(b",").unwrap();
        }
    }
}

impl<W: Write> GeomReader for GeoJsonEmitter<'_, W> {
    fn pointxy(&mut self, x: f64, y: f64, idx: usize) {
        self.comma(idx);
        self.out
            .write(&format!("[{},{}]", x, y).as_bytes())
            .unwrap();
    }
    fn point_begin(&mut self, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "Point", "coordinates": "#)
            .unwrap();
    }
    fn point_end(&mut self) {
        self.out.write(b"}").unwrap();
    }
    fn multipoint_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "MultiPoint", "coordinates": ["#)
            .unwrap();
    }
    fn multipoint_end(&mut self) {
        self.out.write(b"]}").unwrap();
    }
    fn line_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "LineString", "coordinates": ["#)
            .unwrap();
    }
    fn line_end(&mut self, _idx: usize) {
        self.out.write(b"]}").unwrap();
    }
    fn multiline_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "MultiLineString", "coordinates": ["#)
            .unwrap();
    }
    fn multiline_end(&mut self) {
        self.out.write(b"]}").unwrap();
    }
    fn ring_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out.write(b"[").unwrap();
    }
    fn ring_end(&mut self, _idx: usize) {
        self.out.write(b"]").unwrap();
    }
    fn poly_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "Polygon", "coordinates": ["#)
            .unwrap();
    }
    fn poly_end(&mut self, _idx: usize) {
        self.out.write(b"]").unwrap();
    }
    fn subpoly_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out.write(b"[").unwrap();
    }
    fn subpoly_end(&mut self, _idx: usize) {
        self.out.write(b"]").unwrap();
    }
    fn multipoly_begin(&mut self, _size: usize, idx: usize) {
        self.comma(idx);
        self.out
            .write(br#"{"type": "MultiPolygon", "coordinates": ["#)
            .unwrap();
    }
    fn multipoly_end(&mut self) {
        self.out.write(b"]}").unwrap();
    }
}

impl Geometry<'_> {
    pub fn to_geojson<'a, W: Write>(&self, mut out: &'a mut W, geometry_type: GeometryType) {
        let mut json = GeoJsonEmitter::new(&mut out);
        read_geometry(&mut json, self, geometry_type);
    }
}

fn write_num_prop<'a, W: Write>(out: &'a mut W, colname: &String, v: &dyn Display) -> usize {
    out.write(&format!(r#""{}": {}"#, colname, v).as_bytes())
        .unwrap()
}

fn write_str_prop<'a, W: Write>(out: &'a mut W, colname: &String, v: &dyn Display) -> usize {
    out.write(&format!(r#""{}": "{}""#, colname, v).as_bytes())
        .unwrap()
}

impl Feature<'_> {
    pub fn to_geojson<'a, W: Write>(
        &self,
        mut out: &'a mut W,
        columns_meta: &Vec<ColumnMeta>,
        geometry_type: GeometryType,
    ) {
        out.write(br#"{"type": "Feature", "properties": {"#)
            .unwrap();
        let _ = read_properties(self, &columns_meta, |i, colname, colval| {
            if i > 0 {
                out.write(b", ").unwrap();
            }
            match colval {
                ColumnValue::Byte(v) => write_num_prop(out, colname, &v),
                ColumnValue::UByte(v) => write_num_prop(out, colname, &v),
                ColumnValue::Bool(v) => write_num_prop(out, colname, &v),
                ColumnValue::Short(v) => write_num_prop(out, colname, &v),
                ColumnValue::UShort(v) => write_num_prop(out, colname, &v),
                ColumnValue::Int(v) => write_num_prop(out, colname, &v),
                ColumnValue::UInt(v) => write_num_prop(out, colname, &v),
                ColumnValue::Long(v) => write_num_prop(out, colname, &v),
                ColumnValue::ULong(v) => write_num_prop(out, colname, &v),
                ColumnValue::Float(v) => write_num_prop(out, colname, &v),
                ColumnValue::Double(v) => write_num_prop(out, colname, &v),
                ColumnValue::String(v) => write_str_prop(out, colname, &v),
                ColumnValue::Json(_v) => 0,
                ColumnValue::DateTime(v) => write_str_prop(out, colname, &v),
                ColumnValue::Binary(_v) => 0,
            };
            false
        });
        out.write(br#"}, "geometry": "#).unwrap();
        let mut json = GeoJsonEmitter::new(&mut out);
        let geometry = self.geometry().unwrap();
        read_geometry(&mut json, &geometry, geometry_type);
        out.write(b"}").unwrap();
    }
}

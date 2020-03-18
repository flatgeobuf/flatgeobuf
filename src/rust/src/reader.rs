use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom};
use std::mem::size_of;
use std::str;

/// FlatGeobuf header reader
pub struct HeaderReader {
    header_buf: Vec<u8>,
}

impl HeaderReader {
    pub fn read<R: Read + Seek>(mut reader: R) -> std::result::Result<Self, std::io::Error> {
        let mut magic_buf: [u8; 8] = [0; 8];
        reader.read_exact(&mut magic_buf)?;
        if magic_buf != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }

        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf);

        let mut data = HeaderReader {
            header_buf: Vec::with_capacity(header_size as usize),
        };
        data.header_buf.resize(header_size as usize, 0);
        reader.read_exact(&mut data.header_buf)?;

        Ok(data)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.header_buf[..])
    }
}

/// FlatGeobuf feature reader
pub struct FeatureReader {
    feature_base: u64,
    feature_buf: Vec<u8>,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Current position in item_filter
    filter_idx: usize,
}

impl FeatureReader {
    /// Skip R-Tree index
    pub fn select_all<R: Read + Seek>(
        mut reader: R,
        header: &Header,
    ) -> std::result::Result<Self, std::io::Error> {
        let mut data = FeatureReader {
            feature_base: 0,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        };
        // Skip index
        let index_size = PackedRTree::index_size(header.features_count(), header.index_node_size());
        data.feature_base = reader.seek(SeekFrom::Current(index_size as i64))?;
        Ok(data)
    }
    /// Read R-Tree index and build filter for features within bbox
    pub fn select_bbox<R: Read + Seek>(
        mut reader: R,
        header: &Header,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> std::result::Result<Self, std::io::Error> {
        let mut data = FeatureReader {
            feature_base: 0,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        };
        let tree = PackedRTree::from_buf(
            &mut reader,
            header.features_count(),
            PackedRTree::DEFAULT_NODE_SIZE,
        );
        let mut list = tree.search(min_x, min_y, max_x, max_y);
        list.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        data.item_filter = Some(list);
        data.feature_base = reader.seek(SeekFrom::Current(0))?;
        Ok(data)
    }
    /// Number of selected features
    pub fn filter_count(&self) -> Option<usize> {
        self.item_filter.as_ref().map(|f| f.len())
    }
    /// Read next feature
    pub fn next<R: Read + Seek>(
        &mut self,
        mut reader: R,
    ) -> std::result::Result<Feature, std::io::Error> {
        // impl Iterator for Reader is diffcult, because of Feature lifetime
        if let Some(filter) = &self.item_filter {
            if self.filter_idx >= filter.len() {
                return Err(Error::new(ErrorKind::Other, "No more features"));
            }
            let item = &filter[self.filter_idx];
            reader.seek(SeekFrom::Start(self.feature_base + item.offset as u64))?;
            self.filter_idx += 1;
        }
        let mut size_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut size_buf)?;
        let feature_size = u32::from_le_bytes(size_buf);
        self.feature_buf.resize(feature_size as usize, 0);
        reader.read_exact(&mut self.feature_buf)?;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
    }
    /// Return current feature
    pub fn cur_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
}

pub struct ColumnMeta {
    pub coltype: ColumnType,
    pub name: String,
    pub index: usize,
}

fn columns_meta(header: &Header) -> Vec<ColumnMeta> {
    if let Some(columns) = header.columns() {
        columns
            .iter()
            .enumerate()
            .map(|(i, col)| ColumnMeta {
                coltype: col.type_(),
                name: col.name().to_string(),
                index: i,
            })
            .collect()
    } else {
        Vec::new()
    }
}

impl Header<'_> {
    pub fn columns_meta(&self) -> Vec<ColumnMeta> {
        columns_meta(self)
    }
}

// --- Geometry reader ---

pub struct Dimensions {
    /// height
    pub z: bool,
    /// measurement
    pub m: bool,
    /// geodetic decimal year time
    pub t: bool,
    /// time nanosecond measurement
    pub tm: bool,
}

pub trait GeomReader {
    /// Additional dimensions requested from reader
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            z: false,
            m: false,
            t: false,
            tm: false,
        }
    }
    /// Point without additional dimensions
    fn pointxy(&mut self, _x: f64, _y: f64, _idx: usize) {}
    /// Point with additional dimensions
    fn point(
        &mut self,
        _x: f64,
        _y: f64,
        _z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) {
    }
    fn point_begin(&mut self, _idx: usize) {}
    fn point_end(&mut self) {}
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) {}
    fn multipoint_end(&mut self) {}
    fn line_begin(&mut self, _size: usize, _idx: usize) {}
    fn line_end(&mut self, _idx: usize) {}
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {}
    fn multiline_end(&mut self) {}
    fn ring_begin(&mut self, _size: usize, _idx: usize) {}
    fn ring_end(&mut self, _idx: usize) {}
    fn poly_begin(&mut self, _size: usize, _idx: usize) {}
    fn poly_end(&mut self, _idx: usize) {}
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {}
    fn subpoly_end(&mut self, _idx: usize) {}
    fn multipoly_begin(&mut self, _size: usize, _idx: usize) {}
    fn multipoly_end(&mut self) {}
}

pub fn is_collection(geometry_type: GeometryType) -> bool {
    match geometry_type {
        GeometryType::Point
        | GeometryType::MultiPoint
        | GeometryType::LineString
        | GeometryType::MultiLineString
        | GeometryType::Polygon => false,
        GeometryType::MultiPolygon | GeometryType::GeometryCollection => true,
        _ => panic!("is_collection: Unknown geometry type"),
    }
}

fn multi_dim<R: GeomReader>(reader: &mut R) -> bool {
    reader.dimensions().z
        || reader.dimensions().m
        || reader.dimensions().t
        || reader.dimensions().tm
}

fn read_point_multi_dim<R: GeomReader>(
    reader: &mut R,
    geometry: &Geometry,
    offset: usize,
    idx: usize,
) {
    let xy = geometry.xy().unwrap();
    let z = if reader.dimensions().z {
        Some(geometry.z().unwrap().get(offset))
    } else {
        None
    };
    let m = if reader.dimensions().m {
        Some(geometry.m().unwrap().get(offset))
    } else {
        None
    };
    let t = if reader.dimensions().t {
        Some(geometry.t().unwrap().get(offset))
    } else {
        None
    };
    let tm = if reader.dimensions().tm {
        Some(geometry.tm().unwrap().get(offset))
    } else {
        None
    };
    reader.point(xy.get(offset * 2), xy.get(offset * 2 + 1), z, m, t, tm, idx);
}

fn read_points<R: GeomReader>(reader: &mut R, geometry: &Geometry, offset: usize, length: usize) {
    let xy = geometry.xy().unwrap();
    let multi = multi_dim(reader);
    for i in (offset..offset + length).step_by(2) {
        if multi {
            read_point_multi_dim(reader, geometry, i / 2, (i - offset) / 2);
        } else {
            reader.pointxy(xy.get(i), xy.get(i + 1), (i - offset) / 2);
        }
    }
}

fn read_multi_line_part<R: GeomReader>(
    reader: &mut R,
    geometry: &Geometry,
    offset: usize,
    length: usize,
    idx: usize,
) {
    reader.ring_begin(length / 2, idx);
    read_points(reader, geometry, offset, length);
    reader.ring_end(idx);
}

fn read_multi_line<R: GeomReader>(reader: &mut R, geometry: &Geometry, idx: usize) {
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        if let Some(xy) = geometry.xy() {
            reader.multiline_begin(1, idx);
            read_multi_line_part(reader, geometry, 0, xy.len(), 0);
            reader.multiline_end();
        }
    } else {
        let ends = geometry.ends().unwrap();
        reader.multiline_begin(ends.len() / 2, idx);
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            read_multi_line_part(
                reader,
                geometry,
                offset as usize,
                (end - offset) as usize,
                i,
            );
            offset = end;
        }
        reader.multiline_end();
    }
}

fn read_polygon<R: GeomReader>(reader: &mut R, geometry: &Geometry, subpoly: bool, idx: usize) {
    let xy = geometry.xy().unwrap();
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        if subpoly {
            reader.subpoly_begin(1, idx);
        } else {
            reader.poly_begin(1, idx);
        }
        reader.ring_begin(xy.len(), 0);
        read_points(reader, geometry, 0, xy.len());
        reader.ring_end(0);
        if subpoly {
            reader.subpoly_end(idx);
        } else {
            reader.poly_end(idx);
        }
    } else {
        let ends = geometry.ends().unwrap();
        if subpoly {
            reader.subpoly_begin(ends.len() / 2, idx);
        } else {
            reader.poly_begin(ends.len() / 2, idx);
        }
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            let length = (end - offset) as usize;
            reader.ring_begin(length / 2, i);
            read_points(reader, geometry, offset as usize, length);
            reader.ring_end(i);
            offset = end;
        }
        if subpoly {
            reader.subpoly_end(idx);
        } else {
            reader.poly_end(idx);
        }
    }
}

fn read_multi_polygon<R: GeomReader>(reader: &mut R, geometry: &Geometry) {
    let parts = geometry.parts().unwrap();
    reader.multipoly_begin(parts.len(), 0);
    for i in 0..parts.len() {
        let part = parts.get(i);
        read_polygon(reader, &part, true, i);
    }
    reader.multipoly_end();
}

pub fn read_geometry<R: GeomReader>(
    reader: &mut R,
    geometry: &Geometry,
    geometry_type: GeometryType,
) {
    if !is_collection(geometry_type) {
        let xy = geometry.xy().unwrap();
        match geometry_type {
            GeometryType::Point => {
                reader.point_begin(0);
                if multi_dim(reader) {
                    read_point_multi_dim(reader, geometry, 0, 0);
                } else {
                    reader.pointxy(xy.get(0), xy.get(1), 0);
                }
                reader.point_end();
            }
            GeometryType::MultiPoint => {
                reader.multipoint_begin(xy.len() / 2, 0);
                read_points(reader, geometry, 0, xy.len());
                reader.multipoint_end();
            }
            GeometryType::LineString => {
                reader.line_begin(xy.len() / 2, 0);
                read_points(reader, geometry, 0, xy.len());
                reader.line_end(0);
            }
            GeometryType::MultiLineString => {
                read_multi_line(reader, geometry, 0);
            }
            GeometryType::Polygon => {
                read_polygon(reader, geometry, false, 0);
            }
            _ => panic!("read_geometry: Unknown geometry type"),
        }
    }
    match geometry_type {
        GeometryType::MultiPolygon => {
            read_multi_polygon(reader, geometry);
        }
        _ => {} // panic!("read_geometry: Unknown geometry type"),
    }
}

// --- Property reader ---

#[derive(PartialEq, Debug)]
pub enum ColumnValue<'a> {
    Byte(i8),
    UByte(u8),
    Bool(bool),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
    Float(f32),
    Double(f64),
    String(&'a str),
    Json(&'a str),
    DateTime(&'a str),
    Binary(&'a [u8]),
}

fn iter_properties<R>(feature: &Feature, header: &Header, mut reader: R) -> bool
where
    R: FnMut(usize, &str, ColumnValue) -> bool,
{
    let columns_meta = header.columns().unwrap();
    let mut finish = false;
    if let Some(properties) = feature.properties() {
        let mut offset = 0;
        while offset < properties.len() && !finish {
            let i = LittleEndian::read_u16(&properties[offset..offset + 2]) as usize;
            offset += size_of::<u16>();
            let column = &columns_meta.get(i);
            match column.type_() {
                ColumnType::Int => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::Int(LittleEndian::read_i32(&properties[offset..offset + 4])),
                    );
                    offset += size_of::<i32>();
                }
                ColumnType::Long => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::Long(LittleEndian::read_i64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<i64>();
                }
                ColumnType::ULong => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::ULong(LittleEndian::read_u64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<u64>();
                }
                ColumnType::Double => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::Double(LittleEndian::read_f64(
                            &properties[offset..offset + 8],
                        )),
                    );
                    offset += size_of::<f64>();
                }
                ColumnType::String => {
                    let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                    offset += size_of::<u32>();
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::String(
                            // unsafe variant without UTF-8 checking would be faster...
                            str::from_utf8(&properties[offset..offset + len])
                                .expect("Invalid UTF-8 string"),
                        ),
                    );
                    offset += len;
                }
                ColumnType::Byte => todo!(),
                ColumnType::UByte => todo!(),
                ColumnType::Bool => todo!(),
                ColumnType::Short => todo!(),
                ColumnType::UShort => todo!(),
                ColumnType::UInt => todo!(),
                ColumnType::Float => todo!(),
                ColumnType::Json => todo!(),
                ColumnType::DateTime => todo!(),
                ColumnType::Binary => todo!(),
            }
        }
    }
    finish
}

fn properties_map(feature: &Feature, header: &Header) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let _ = iter_properties(&feature, &header, |_i, colname, colval| {
        let vstr = match colval {
            ColumnValue::Byte(v) => format!("{}", v),
            ColumnValue::UByte(v) => format!("{}", v),
            ColumnValue::Bool(v) => format!("{}", v),
            ColumnValue::Short(v) => format!("{}", v),
            ColumnValue::UShort(v) => format!("{}", v),
            ColumnValue::Int(v) => format!("{}", v),
            ColumnValue::UInt(v) => format!("{}", v),
            ColumnValue::Long(v) => format!("{}", v),
            ColumnValue::ULong(v) => format!("{}", v),
            ColumnValue::Float(v) => format!("{}", v),
            ColumnValue::Double(v) => format!("{}", v),
            ColumnValue::String(v) => format!("{}", v),
            ColumnValue::Json(v) => format!("{}", v),
            ColumnValue::DateTime(v) => format!("{}", v),
            ColumnValue::Binary(_v) => "[BINARY]".to_string(),
        };
        properties.insert(colname.to_string(), vstr);
        false
    });
    properties
}

impl Feature<'_> {
    pub fn iter_properties<R>(&self, header: &Header, reader: R) -> bool
    where
        R: FnMut(usize, &str, ColumnValue) -> bool,
    {
        iter_properties(self, &header, reader)
    }
    pub fn properties_map(&self, header: &Header) -> HashMap<String, String> {
        properties_map(self, &header)
    }
}

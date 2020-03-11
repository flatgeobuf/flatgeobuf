use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::packed_r_tree::{self, PackedRTree};
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::mem::size_of;
use std::str;

pub struct Reader<R: Read> {
    reader: BufReader<R>,
    header_buf: Vec<u8>,
    feature_base: u64,
    feature_buf: Vec<u8>,
    /// Selected features or None if no bbox filter
    item_filter: Option<Vec<packed_r_tree::SearchResultItem>>,
    /// Current position in item_filter
    filter_idx: usize,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(reader: R) -> Reader<R> {
        Reader {
            reader: BufReader::new(reader),
            header_buf: Vec::new(),
            feature_base: 0,
            feature_buf: Vec::new(),
            item_filter: None,
            filter_idx: 0,
        }
    }
    pub fn read_header(&mut self) -> std::result::Result<Header, std::io::Error> {
        let mut magic_buf: [u8; 8] = [0; 8];
        self.reader.read_exact(&mut magic_buf)?;
        if magic_buf != MAGIC_BYTES {
            return Err(Error::new(ErrorKind::Other, "Magic byte doesn't match"));
        }

        let mut size_buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut size_buf)?;
        let header_size = u32::from_le_bytes(size_buf);

        self.header_buf.resize(header_size as usize, 0);
        self.reader.read_exact(&mut self.header_buf)?;

        let header = get_root_as_header(&self.header_buf[..]);
        Ok(header)
    }
    pub fn header(&self) -> Header {
        get_root_as_header(&self.header_buf[..])
    }
    pub fn select_all(&mut self) -> std::result::Result<(), std::io::Error> {
        let header = get_root_as_header(&self.header_buf[..]);
        // Skip index
        let index_size = PackedRTree::size(header.features_count(), header.index_node_size());
        self.reader.seek(SeekFrom::Current(index_size as i64))?;
        Ok(())
    }
    pub fn select_bbox(
        &mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> std::result::Result<(), std::io::Error> {
        let header = get_root_as_header(&self.header_buf[..]);
        let tree = PackedRTree::from_buf(
            &mut self.reader,
            header.features_count(),
            PackedRTree::DEFAULT_NODE_SIZE,
        );
        let mut list = tree.search(min_x, min_y, max_x, max_y);
        list.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        self.item_filter = Some(list);
        self.feature_base = self.reader.seek(SeekFrom::Current(0))?;
        Ok(())
    }
    pub fn select_count(&self) -> Option<usize> {
        self.item_filter.as_ref().map(|f| f.len())
    }
    pub fn next(&mut self) -> std::result::Result<Feature, std::io::Error> {
        // impl Iterator for Reader is diffcult, because of Feature lifetime
        if let Some(filter) = &self.item_filter {
            if self.filter_idx >= filter.len() {
                return Err(Error::new(ErrorKind::Other, "No more features"));
            }
            let item = &filter[self.filter_idx];
            self.reader
                .seek(SeekFrom::Start(self.feature_base + item.offset as u64))?;
            self.filter_idx += 1;
        }
        let mut size_buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut size_buf)?;
        let feature_size = u32::from_le_bytes(size_buf);
        self.feature_buf.resize(feature_size as usize, 0);
        self.reader.read_exact(&mut self.feature_buf)?;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
    }
    pub fn cur_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
}

pub struct Dimensions {
    pub z: bool,
    pub m: bool,
    pub t: bool,
    pub tm: bool,
}

pub trait GeomReader {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            z: false,
            m: false,
            t: false,
            tm: false,
        }
    }
    fn pointxy(&mut self, _x: f64, _y: f64) {}
    fn point(
        &mut self,
        _x: f64,
        _y: f64,
        _z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
    ) {
    }
    fn multipoint_begin(&mut self, _size: usize) {}
    fn multipoint_end(&mut self) {}
    fn line_begin(&mut self, _size: usize) {}
    fn line_end(&mut self) {}
    fn multiline_begin(&mut self, _size: usize) {}
    fn multiline_end(&mut self) {}
    fn ring_begin(&mut self, _size: usize) {}
    fn ring_end(&mut self) {}
    fn poly_begin(&mut self, _size: usize) {}
    fn poly_end(&mut self) {}
    fn multipoly_begin(&mut self, _size: usize) {}
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

fn read_point_multi_dim<R: GeomReader>(reader: &mut R, geometry: &Geometry, offset: usize) {
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
    reader.point(xy.get(offset * 2), xy.get(offset * 2 + 1), z, m, t, tm);
}

fn read_points<R: GeomReader>(reader: &mut R, geometry: &Geometry, offset: usize, length: usize) {
    let xy = geometry.xy().unwrap();
    let multi = multi_dim(reader);
    for i in (offset..offset + length).step_by(2) {
        if multi {
            read_point_multi_dim(reader, geometry, i / 2);
        } else {
            reader.pointxy(xy.get(i), xy.get(i + 1));
        }
    }
}

fn read_line<R: GeomReader>(reader: &mut R, geometry: &Geometry, offset: usize, length: usize) {
    reader.line_begin(length / 2);
    read_points(reader, geometry, offset, length);
    reader.line_end();
}

pub fn read_multi_line<R: GeomReader>(reader: &mut R, geometry: &Geometry) {
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        if let Some(xy) = geometry.xy() {
            reader.multiline_begin(1);
            read_line(reader, geometry, 0, xy.len());
            reader.multiline_end();
        }
    } else {
        let ends = geometry.ends().unwrap();
        reader.multiline_begin(ends.len() / 2);
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            read_line(reader, geometry, offset as usize, (end - offset) as usize);
            offset = end;
        }
        reader.multiline_end();
    }
}

pub fn read_polygon<R: GeomReader>(reader: &mut R, geometry: &Geometry) {
    let xy = geometry.xy().unwrap();
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        reader.poly_begin(xy.len());
        read_points(reader, geometry, 0, xy.len());
    } else {
        let ends = geometry.ends().unwrap();
        reader.poly_begin(ends.len() / 2);
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            let length = (end - offset) as usize;
            reader.ring_begin(length / 2);
            read_points(reader, geometry, offset as usize, length);
            reader.ring_end();
            offset = end;
        }
    }
    reader.poly_end();
}

pub fn read_multi_polygon<R: GeomReader>(reader: &mut R, geometry: &Geometry) {
    let parts = geometry.parts().unwrap();
    reader.multipoly_begin(parts.len());
    for i in 0..parts.len() {
        let part = parts.get(i);
        read_polygon(reader, &part);
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
                if multi_dim(reader) {
                    read_point_multi_dim(reader, geometry, 0);
                } else {
                    reader.pointxy(xy.get(0), xy.get(1));
                }
            }
            GeometryType::MultiPoint => {
                reader.multipoint_begin(xy.len() / 2);
                read_points(reader, geometry, 0, xy.len());
                reader.multipoint_end();
            }
            GeometryType::LineString => {
                read_line(reader, geometry, 0, xy.len());
            }
            GeometryType::MultiLineString => {
                read_multi_line(reader, geometry);
            }
            GeometryType::Polygon => {
                read_polygon(reader, geometry);
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

pub struct ColumnMeta {
    pub coltype: ColumnType,
    pub name: String,
    pub index: usize,
}

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

pub fn columns_meta(header: &Header) -> Vec<ColumnMeta> {
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

pub fn read_properties<R>(feature: &Feature, columns_meta: &Vec<ColumnMeta>, mut reader: R) -> bool
where
    R: FnMut(usize, &String, ColumnValue) -> bool,
{
    let mut finish = false;
    if let Some(properties) = feature.properties() {
        let mut offset = 0;
        while offset < properties.len() && !finish {
            let i = LittleEndian::read_u16(&properties[offset..offset + 2]) as usize;
            offset += size_of::<u16>();
            let column = &columns_meta[i];
            match column.coltype {
                ColumnType::Int => {
                    finish = reader(
                        i,
                        &column.name,
                        ColumnValue::Int(LittleEndian::read_i32(&properties[offset..offset + 4])),
                    );
                    offset += size_of::<i32>();
                }
                ColumnType::Long => {
                    finish = reader(
                        i,
                        &column.name,
                        ColumnValue::Long(LittleEndian::read_i64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<i64>();
                }
                ColumnType::ULong => {
                    finish = reader(
                        i,
                        &column.name,
                        ColumnValue::ULong(LittleEndian::read_u64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<u64>();
                }
                ColumnType::Double => {
                    finish = reader(
                        i,
                        &column.name,
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
                        &column.name,
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

pub fn read_all_properties(
    feature: &Feature,
    columns_meta: &Vec<ColumnMeta>,
) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let _ = read_properties(&feature, &columns_meta, |_i, colname, colval| {
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

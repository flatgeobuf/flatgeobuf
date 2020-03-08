use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::MAGIC_BYTES;
use byteorder::{ByteOrder, LittleEndian};
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::mem::size_of;

pub struct Reader<R: Read> {
    reader: BufReader<R>,
    header_buf: Vec<u8>,
    feature_buf: Vec<u8>,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(reader: R) -> Reader<R> {
        Reader {
            reader: BufReader::new(reader),
            header_buf: Vec::new(),
            feature_buf: Vec::new(),
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
    pub fn select_all(&mut self) -> std::result::Result<(), std::io::Error> {
        let header = get_root_as_header(&self.header_buf[..]);
        // Skip index
        let index_size = packed_rtree_size(header.features_count(), header.index_node_size());
        self.reader.seek(SeekFrom::Current(index_size as i64))?;
        Ok(())
    }
    pub fn next(&mut self) -> std::result::Result<Feature, std::io::Error> {
        // impl Iterator for Reader is diffcult, because Type Feature has a lifetime
        let mut size_buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut size_buf)?;
        let feature_size = u32::from_le_bytes(size_buf);
        self.feature_buf.resize(feature_size as usize, 0);
        self.reader.read_exact(&mut self.feature_buf)?;
        let feature = get_root_as_feature(&self.feature_buf[..]);
        Ok(feature)
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

pub fn read_line<R: GeomReader>(reader: &mut R, geometry: &Geometry, offset: usize, length: usize) {
    reader.line_begin(length / 2);
    read_points(reader, geometry, offset, length);
    reader.line_end();
}

pub fn read_multi_line<R: GeomReader>(reader: &mut R, geometry: &Geometry) {
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        if let Some(xy) = geometry.xy() {
            reader.multiline_begin(1);
            read_line(reader, geometry, 0, xy.len());
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
    }
    reader.multiline_end();
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
pub enum ColumnValue {
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
    String(String),
    Json(String),
    DateTime(String),
    Binary(Vec<u8>),
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

pub fn property_values(
    feature: &Feature,
    columns_meta: &Vec<ColumnMeta>,
) -> Vec<(usize, ColumnValue)> {
    let mut propvalues = Vec::new();
    if let Some(properties) = feature.properties() {
        let mut offset = 0;
        while offset < properties.len() {
            let i = LittleEndian::read_u16(&properties[offset..offset + 2]) as usize;
            offset += size_of::<u16>();
            let column = &columns_meta[i];
            match column.coltype {
                ColumnType::Int => {
                    propvalues.push((
                        i,
                        ColumnValue::Int(LittleEndian::read_i32(&properties[offset..offset + 4])),
                    ));
                    offset += size_of::<i32>();
                }
                ColumnType::Long => {
                    propvalues.push((
                        i,
                        ColumnValue::Long(LittleEndian::read_i64(&properties[offset..offset + 8])),
                    ));
                    offset += size_of::<i64>();
                }
                ColumnType::ULong => {
                    propvalues.push((
                        i,
                        ColumnValue::ULong(LittleEndian::read_u64(&properties[offset..offset + 8])),
                    ));
                    offset += size_of::<u64>();
                }
                ColumnType::Double => {
                    propvalues.push((
                        i,
                        ColumnValue::Double(LittleEndian::read_f64(
                            &properties[offset..offset + 8],
                        )),
                    ));
                    offset += size_of::<f64>();
                }
                ColumnType::String => {
                    let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                    offset += size_of::<u32>();
                    propvalues.push((
                        i,
                        ColumnValue::String(
                            String::from_utf8_lossy(&properties[offset..offset + len]).to_string(),
                        ),
                    ));
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
    propvalues
}

pub fn packed_rtree_size(num_items: u64, node_size: u16) -> u64 {
    let node_size_min = node_size as u64;
    let mut n = num_items;
    let mut num_nodes = n;
    loop {
        n = (n + node_size_min - 1) / node_size_min;
        num_nodes += n;
        if n == 1 {
            break;
        }
    }
    num_nodes * 40
}
// uint64_t PackedRTree::size(const uint64_t numItems, const uint16_t nodeSize)
// {
//     if (nodeSize < 2)
//         throw std::invalid_argument("Node size must be at least 2");
//     if (numItems == 0)
//         throw std::invalid_argument("Number of items must be greater than 0");
//     const uint16_t nodeSizeMin = std::min(std::max(nodeSize, static_cast<uint16_t>(2)), static_cast<uint16_t>(65535));
//     // limit so that resulting size in bytes can be represented by uint64_t
//     if (numItems > static_cast<uint64_t>(1) << 56)
//         throw std::overflow_error("Number of items must be less than 2^56");
//     uint64_t n = numItems;
//     uint64_t numNodes = n;
//     do {
//         n = (n + nodeSizeMin - 1) / nodeSizeMin;
//         numNodes += n;
//     } while (n != 1);
//     return numNodes * sizeof(NodeItem);
// }

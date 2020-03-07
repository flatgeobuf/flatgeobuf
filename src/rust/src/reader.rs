use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use crate::MAGIC_BYTES;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};

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

pub trait GeomVisitor {
    fn pointxy(&mut self, _x: f64, _y: f64) {}
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

fn visit_points<F: GeomVisitor>(
    visitor: &mut F,
    xy: flatbuffers::Vector<f64>,
    offset: usize,
    length: usize,
) {
    for i in (offset..offset + length).step_by(2) {
        visitor.pointxy(xy.get(i), xy.get(i + 1));
    }
}

pub fn visit_line<F: GeomVisitor>(
    visitor: &mut F,
    xy: flatbuffers::Vector<f64>,
    offset: usize,
    length: usize,
) {
    visitor.line_begin(length / 2);
    visit_points(visitor, xy, offset, length);
    visitor.line_end();
}

pub fn visit_multi_line<F: GeomVisitor>(visitor: &mut F, geometry: &Geometry) {
    let xy = geometry.xy().unwrap();
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        visitor.multiline_begin(1);
        visit_line(visitor, xy, 0, xy.len());
    } else {
        let ends = geometry.ends().unwrap();
        visitor.multiline_begin(ends.len() / 2);
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            visit_line(visitor, xy, offset as usize, (end - offset) as usize);
            offset = end;
        }
    }
    visitor.multiline_end();
}

pub fn visit_polygon<F: GeomVisitor>(visitor: &mut F, geometry: &Geometry) {
    let xy = geometry.xy().unwrap();
    if geometry.ends().is_none() || geometry.ends().unwrap().len() < 2 {
        visitor.poly_begin(xy.len());
        visit_points(visitor, xy, 0, xy.len());
    } else {
        let ends = geometry.ends().unwrap();
        visitor.poly_begin(ends.len() / 2);
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            let length = (end - offset) as usize;
            visitor.ring_begin(length / 2);
            visit_points(visitor, xy, offset as usize, length);
            visitor.ring_end();
            offset = end;
        }
    }
    visitor.poly_end();
}

pub fn visit_multi_polygon<F: GeomVisitor>(visitor: &mut F, geometry: &Geometry) {
    let parts = geometry.parts().unwrap();
    visitor.multipoly_begin(parts.len());
    for i in 0..parts.len() {
        let part = parts.get(i);
        visit_polygon(visitor, &part);
    }
    visitor.multipoly_end();
}

pub fn visit_geometry<F: GeomVisitor>(
    visitor: &mut F,
    geometry: &Geometry,
    geometry_type: GeometryType,
) {
    if !is_collection(geometry_type) {
        let xy = geometry.xy().unwrap();
        match geometry_type {
            GeometryType::Point => {
                visitor.pointxy(xy.get(0), xy.get(1));
            }
            GeometryType::MultiPoint => {
                visitor.multipoint_begin(xy.len() / 2);
                visit_points(visitor, xy, 0, xy.len());
                visitor.multipoint_end();
            }
            GeometryType::LineString => {
                visit_line(visitor, xy, 0, xy.len());
            }
            GeometryType::MultiLineString => {
                visit_multi_line(visitor, geometry);
            }
            GeometryType::Polygon => {
                visit_polygon(visitor, geometry);
            }
            _ => panic!("visit_geometry: Unknown geometry type"),
        }
    }
    match geometry_type {
        GeometryType::MultiPolygon => {
            visit_multi_polygon(visitor, geometry);
        }
        _ => {} // panic!("visit_geometry: Unknown geometry type"),
    }
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

use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;

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

pub struct DebugReader;

impl GeomReader for DebugReader {
    fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
        print!("pointxy({} {}) ", x, y);
    }
    fn point_begin(&mut self, _idx: usize) {
        print!("point_begin ");
    }
    fn point_end(&mut self) {
        println!("point_end ");
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) {
        print!("multipoint_begin ");
    }
    fn multipoint_end(&mut self) {
        println!("multipoint_end ");
    }
    fn line_begin(&mut self, _size: usize, _idx: usize) {
        print!("line_begin ");
    }
    fn line_end(&mut self, _idx: usize) {
        println!("line_end ");
    }
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {
        print!("multiline_begin ");
    }
    fn multiline_end(&mut self) {
        println!("multiline_end ");
    }
    fn ring_begin(&mut self, _size: usize, _idx: usize) {
        print!("ring_begin ");
    }
    fn ring_end(&mut self, _idx: usize) {
        println!("ring_end ");
    }
    fn poly_begin(&mut self, _size: usize, _idx: usize) {
        print!("poly_begin ");
    }
    fn poly_end(&mut self, _idx: usize) {
        println!("poly_end ");
    }
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {
        print!("subpoly_begin ");
    }
    fn subpoly_end(&mut self, _idx: usize) {
        println!("subpoly_end ");
    }
    fn multipoly_begin(&mut self, _size: usize, _idx: usize) {
        print!("multipoly_begin ");
    }
    fn multipoly_end(&mut self) {
        println!("multipoly_end ");
    }
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

impl Geometry<'_> {
    pub fn parse<R: GeomReader>(&self, reader: &mut R, geometry_type: GeometryType) {
        read_geometry(reader, self, geometry_type);
    }
}

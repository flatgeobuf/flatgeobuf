use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use geozero::error::{GeozeroError, Result};
use geozero::GeomProcessor;

pub fn is_collection(geometry_type: GeometryType) -> Result<bool> {
    let coll = match geometry_type {
        GeometryType::Point
        | GeometryType::MultiPoint
        | GeometryType::LineString
        | GeometryType::MultiLineString
        | GeometryType::Polygon => false,
        GeometryType::MultiPolygon | GeometryType::GeometryCollection => true,
        _ => Err(GeozeroError::Geometry(
            "is_collection: Unknown geometry type".to_string(),
        ))?,
    };
    Ok(coll)
}

fn multi_dim<P: GeomProcessor>(processor: &mut P) -> bool {
    processor.dimensions().z
        || processor.dimensions().m
        || processor.dimensions().t
        || processor.dimensions().tm
}

fn read_coordinate<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    offset: usize,
    idx: usize,
) -> Result<()> {
    let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
    let z = if processor.dimensions().z {
        geometry.z().and_then(|dim| Some(dim.get(offset)))
    } else {
        None
    };
    let m = if processor.dimensions().m {
        geometry.m().and_then(|dim| Some(dim.get(offset)))
    } else {
        None
    };
    let t = if processor.dimensions().t {
        geometry.t().and_then(|dim| Some(dim.get(offset)))
    } else {
        None
    };
    let tm = if processor.dimensions().tm {
        geometry.tm().and_then(|dim| Some(dim.get(offset)))
    } else {
        None
    };
    processor.coordinate(xy.get(offset * 2), xy.get(offset * 2 + 1), z, m, t, tm, idx)
}

fn read_coords<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    offset: usize,
    length: usize,
) -> Result<()> {
    let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
    let multi = multi_dim(processor);
    for i in (offset..offset + length).step_by(2) {
        if multi {
            read_coordinate(processor, geometry, i / 2, (i - offset) / 2)?;
        } else {
            processor.xy(xy.get(i), xy.get(i + 1), (i - offset) / 2)?;
        }
    }
    Ok(())
}

fn read_multilinestring_part<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    offset: usize,
    length: usize,
    idx: usize,
) -> Result<()> {
    processor.linestring_begin(false, length / 2, idx)?;
    read_coords(processor, geometry, offset, length)?;
    processor.linestring_end(false, idx)
}

fn read_multiline<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() < 2 {
        if let Some(xy) = geometry.xy() {
            processor.multilinestring_begin(1, idx)?;
            read_multilinestring_part(processor, geometry, 0, xy.len(), 0)?;
            processor.multilinestring_end(idx)?;
        }
    } else {
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.multilinestring_begin(ends.len() / 2, idx)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            read_multilinestring_part(
                processor,
                geometry,
                offset as usize,
                (end - offset) as usize,
                i,
            )?;
            offset = end;
        }
        processor.multilinestring_end(idx)?;
    }
    Ok(())
}

fn read_polygon<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    tagged: bool,
    idx: usize,
) -> Result<()> {
    let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() < 2 {
        processor.polygon_begin(tagged, 1, idx)?;
        processor.linestring_begin(false, xy.len(), 0)?;
        read_coords(processor, geometry, 0, xy.len())?;
        processor.linestring_end(false, 0)?;
        processor.polygon_end(tagged, idx)?;
    } else {
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.polygon_begin(tagged, ends.len() / 2, idx)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) << 1;
            let length = (end - offset) as usize;
            processor.linestring_begin(false, length / 2, i)?;
            read_coords(processor, geometry, offset as usize, length)?;
            processor.linestring_end(false, i)?;
            offset = end;
        }
        processor.polygon_end(tagged, idx)?;
    }
    Ok(())
}

fn read_multi_polygon<P: GeomProcessor>(processor: &mut P, geometry: &Geometry) -> Result<()> {
    let parts = geometry.parts().ok_or(GeozeroError::GeometryFormat)?;
    processor.multipolygon_begin(parts.len(), 0)?;
    for i in 0..parts.len() {
        let part = parts.get(i);
        read_polygon(processor, &part, false, i)?;
    }
    processor.multipolygon_end(0)
}

pub fn read_geometry<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    geometry_type: GeometryType,
) -> Result<()> {
    if !is_collection(geometry_type)? {
        let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
        match geometry_type {
            GeometryType::Point => {
                processor.point_begin(0)?;
                if multi_dim(processor) {
                    read_coordinate(processor, geometry, 0, 0)?;
                } else {
                    processor.xy(xy.get(0), xy.get(1), 0)?;
                }
                processor.point_end(0)?;
            }
            GeometryType::MultiPoint => {
                processor.multipoint_begin(xy.len() / 2, 0)?;
                read_coords(processor, geometry, 0, xy.len())?;
                processor.multipoint_end(0)?;
            }
            GeometryType::LineString => {
                processor.linestring_begin(true, xy.len() / 2, 0)?;
                read_coords(processor, geometry, 0, xy.len())?;
                processor.linestring_end(true, 0)?;
            }
            GeometryType::MultiLineString => {
                read_multiline(processor, geometry, 0)?;
            }
            GeometryType::Polygon => {
                read_polygon(processor, geometry, true, 0)?;
            }
            _ => Err(GeozeroError::Geometry(
                "read_geometry: Unknown geometry type".to_string(),
            ))?,
        }
    }
    match geometry_type {
        GeometryType::MultiPolygon => {
            read_multi_polygon(processor, geometry)?;
        }
        _ => {} // panic!("read_geometry: Unknown geometry type"),
    }
    Ok(())
}

impl Geometry<'_> {
    pub fn process<P: GeomProcessor>(
        &self,
        processor: &mut P,
        geometry_type: GeometryType,
    ) -> Result<()> {
        read_geometry(processor, self, geometry_type)
    }
}

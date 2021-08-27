use crate::feature_generated::*;
use crate::header_generated::*;
use geozero::error::{GeozeroError, Result};
use geozero::GeomProcessor;

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
    let multi = processor.multi_dim();
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

fn read_multilinestring<P: GeomProcessor>(
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

fn read_triangle<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    tagged: bool,
    idx: usize,
) -> Result<()> {
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() < 2 {
        if let Some(xy) = geometry.xy() {
            processor.triangle_begin(tagged, 1, idx)?;
            read_multilinestring_part(processor, geometry, 0, xy.len(), 0)?;
            processor.triangle_end(tagged, idx)?;
        }
    } else {
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            processor.triangle_begin(tagged, 1, i)?;
            let end = ends.get(i) << 1;
            read_multilinestring_part(
                processor,
                geometry,
                offset as usize,
                (end - offset) as usize,
                0,
            )?;
            offset = end;
            processor.triangle_end(tagged, i)?;
        }
    }
    Ok(())
}

fn read_tin<P: GeomProcessor>(processor: &mut P, geometry: &Geometry, idx: usize) -> Result<()> {
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() < 2 {
        processor.tin_begin(1, idx)?;
        read_triangle(processor, geometry, false, 0)?;
        processor.tin_end(idx)?;
    } else {
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.tin_begin(ends.len() / 2, idx)?;
        read_triangle(processor, geometry, false, 0)?;
        processor.tin_end(idx)?;
    }
    Ok(())
}

fn read_polygon<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    tagged: bool,
    idx: usize,
) -> Result<()> {
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() < 2 {
        // single ring
        processor.polygon_begin(tagged, 1, idx)?;
        let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
        processor.linestring_begin(false, xy.len(), 0)?;
        read_coords(processor, geometry, 0, xy.len())?;
        processor.linestring_end(false, 0)?;
        processor.polygon_end(tagged, idx)?;
    } else {
        // multiple rings
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

fn read_curve<P: GeomProcessor>(
    processor: &mut P,
    fn_begin: fn(&mut P, size: usize, idx: usize) -> Result<()>,
    fn_end: fn(&mut P, idx: usize) -> Result<()>,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    let compoundcurve_allowed = geometry.type_() != GeometryType::CompoundCurve;
    let polygon_allowed = geometry.type_() == GeometryType::MultiSurface;
    let parts = geometry.parts().ok_or(GeozeroError::GeometryFormat)?;
    fn_begin(processor, parts.len(), idx)?;
    for i in 0..parts.len() {
        let geometry = parts.get(i);
        let geometry_type = geometry.type_();
        match geometry_type {
            GeometryType::LineString => {
                let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
                processor.linestring_begin(false, xy.len(), i)?;
                read_coords(processor, &geometry, 0, xy.len())?;
                processor.linestring_end(false, i)?;
            }
            GeometryType::CircularString => {
                let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
                processor.circularstring_begin(xy.len() / 2, i)?;
                read_coords(processor, &geometry, 0, xy.len())?;
                processor.circularstring_end(i)?;
            }
            GeometryType::CompoundCurve if compoundcurve_allowed => {
                read_compoundcurve(processor, &geometry, i)?;
            }
            GeometryType::Polygon if polygon_allowed => {
                read_polygon(processor, &geometry, true, idx)?;
            }
            GeometryType::CurvePolygon if polygon_allowed => {
                read_curvepolygon(processor, &geometry, idx)?;
            }
            _ => {
                return Err(GeozeroError::Geometry(format!(
                    "Unexpected geometry type in curve: {:?}",
                    geometry_type
                )))
            }
        }
    }
    fn_end(processor, idx)?;
    Ok(())
}

fn read_compoundcurve<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    read_curve(
        processor,
        GeomProcessor::compoundcurve_begin,
        GeomProcessor::compoundcurve_end,
        geometry,
        idx,
    )
}

fn read_curvepolygon<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    read_curve(
        processor,
        GeomProcessor::curvepolygon_begin,
        GeomProcessor::curvepolygon_end,
        geometry,
        idx,
    )
}

fn read_multipolygon_type<P: GeomProcessor>(
    processor: &mut P,
    fn_begin: fn(&mut P, size: usize, idx: usize) -> Result<()>,
    fn_end: fn(&mut P, idx: usize) -> Result<()>,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    let parts = geometry.parts().ok_or(GeozeroError::GeometryFormat)?;
    fn_begin(processor, parts.len(), idx)?;
    for i in 0..parts.len() {
        let part = parts.get(i);
        read_polygon(processor, &part, false, i)?;
    }
    fn_end(processor, idx)
}
fn read_geometrycollection<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    idx: usize,
) -> Result<()> {
    let parts = geometry.parts().ok_or(GeozeroError::GeometryFormat)?;
    processor.geometrycollection_begin(parts.len(), idx)?;
    for i in 0..parts.len() {
        let part = parts.get(i);
        read_geometry_n(processor, &part, part.type_(), i)?;
    }
    processor.geometrycollection_end(idx)
}

pub fn read_geometry<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    geometry_type: GeometryType,
) -> Result<()> {
    let geometry_type = if geometry_type == GeometryType::Unknown {
        // per feature geometry type
        geometry.type_()
    } else {
        geometry_type
    };
    read_geometry_n(processor, geometry, geometry_type, 0)
}

fn read_geometry_n<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    geometry_type: GeometryType,
    idx: usize,
) -> Result<()> {
    match geometry_type {
        GeometryType::Point => {
            processor.point_begin(idx)?;
            if processor.multi_dim() {
                read_coordinate(processor, geometry, 0, 0)?;
            } else {
                let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
                processor.xy(xy.get(0), xy.get(1), 0)?;
            }
            processor.point_end(idx)?;
        }
        GeometryType::MultiPoint => {
            let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
            processor.multipoint_begin(xy.len() / 2, idx)?;
            read_coords(processor, geometry, 0, xy.len())?;
            processor.multipoint_end(idx)?;
        }
        GeometryType::LineString => {
            let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
            processor.linestring_begin(true, xy.len() / 2, idx)?;
            read_coords(processor, geometry, 0, xy.len())?;
            processor.linestring_end(true, idx)?;
        }
        GeometryType::CircularString => {
            let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
            processor.circularstring_begin(xy.len() / 2, idx)?;
            read_coords(processor, geometry, 0, xy.len())?;
            processor.circularstring_end(idx)?;
        }
        GeometryType::CompoundCurve => {
            read_compoundcurve(processor, geometry, idx)?;
        }
        GeometryType::MultiLineString => {
            read_multilinestring(processor, geometry, idx)?;
        }
        GeometryType::MultiCurve => {
            read_curve(
                processor,
                GeomProcessor::multicurve_begin,
                GeomProcessor::multicurve_end,
                geometry,
                idx,
            )?;
        }
        GeometryType::Polygon => {
            read_polygon(processor, geometry, true, idx)?;
        }
        GeometryType::CurvePolygon => {
            read_curvepolygon(processor, geometry, idx)?;
        }
        GeometryType::MultiPolygon => {
            read_multipolygon_type(
                processor,
                GeomProcessor::multipolygon_begin,
                GeomProcessor::multipolygon_end,
                geometry,
                idx,
            )?;
        }
        GeometryType::PolyhedralSurface => {
            read_multipolygon_type(
                processor,
                GeomProcessor::polyhedralsurface_begin,
                GeomProcessor::polyhedralsurface_end,
                geometry,
                idx,
            )?;
        }
        GeometryType::TIN => {
            read_tin(processor, geometry, idx)?;
        }
        GeometryType::Triangle => {
            read_triangle(processor, geometry, true, idx)?;
        }
        GeometryType::MultiSurface => {
            read_curve(
                processor,
                GeomProcessor::multisurface_begin,
                GeomProcessor::multisurface_end,
                geometry,
                idx,
            )?;
        }
        GeometryType::GeometryCollection => {
            read_geometrycollection(processor, geometry, idx)?;
        }
        _ => {
            return Err(GeozeroError::Geometry(format!(
                "Unknown geometry type {:?}",
                geometry_type
            )))
        }
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

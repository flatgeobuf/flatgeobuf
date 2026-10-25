use crate::feature_generated::*;
use crate::header_generated::*;
use geozero::error::{GeozeroError, Result};
use geozero::GeomProcessor;

/// Read FlatGeobuf geometry
// See https://worace.works/2022/03/12/flatgeobuf-implementers-guide/
// for a format description
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

impl Geometry<'_> {
    /// Stream this FlatGeobuf geometry into a GeoZero [`GeomProcessor`].
    ///
    /// This is a convenience wrapper around [`read_geometry`].
    pub fn process<P: GeomProcessor>(
        &self,
        processor: &mut P,
        geometry_type: GeometryType,
    ) -> Result<()> {
        read_geometry(processor, self, geometry_type)
    }
}

/// Number of coordinate pairs in the geometry's `xy` array.
///
/// Coordinate consumers index `xy` in pairs, so an odd ordinate count (file-controlled and
/// not covered by the FlatBuffers verifier) is rejected instead of read past.
fn xy_point_count(geometry: &Geometry) -> Result<usize> {
    let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
    if xy.len() % 2 != 0 {
        return Err(GeozeroError::Geometry(
            "FlatGeobuf geometry has an odd number of xy ordinates".to_string(),
        ));
    }
    Ok(xy.len() / 2)
}

/// Validate the file-controlled `ends` array against the coordinate count.
///
/// The FlatBuffers verifier bounds `ends` on its own but never against `xy`, so a part end
/// past the last coordinate pair, or an end below its predecessor, would otherwise index
/// past the coordinate vector or underflow the part length.
fn check_ends(geometry: &Geometry) -> Result<()> {
    let point_count = xy_point_count(geometry)?;
    let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
    let mut previous = 0;
    for i in 0..ends.len() {
        let end = ends.get(i) as usize;
        if end > point_count {
            return Err(GeozeroError::Geometry(
                "FlatGeobuf geometry part end exceeds the coordinate count".to_string(),
            ));
        }
        if end < previous {
            return Err(GeozeroError::Geometry(
                "FlatGeobuf geometry part ends are not monotonic".to_string(),
            ));
        }
        previous = end;
    }
    Ok(())
}

/// Read one optional ordinate, rejecting a dimension array shorter than the geometry.
fn dimension_value(
    values: Option<flatbuffers::Vector<'_, f64>>,
    offset: usize,
) -> Result<Option<f64>> {
    match values {
        Some(values) if offset < values.len() => Ok(Some(values.get(offset))),
        Some(_) => Err(GeozeroError::Geometry(
            "FlatGeobuf geometry dimension array is shorter than the coordinate count".to_string(),
        )),
        None => Ok(None),
    }
}

/// As [`dimension_value`], for the `tm` nanosecond array.
fn time_value(values: Option<flatbuffers::Vector<'_, u64>>, offset: usize) -> Result<Option<u64>> {
    match values {
        Some(values) if offset < values.len() => Ok(Some(values.get(offset))),
        Some(_) => Err(GeozeroError::Geometry(
            "FlatGeobuf geometry dimension array is shorter than the coordinate count".to_string(),
        )),
        None => Ok(None),
    }
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
                if xy.len() < 2 {
                    return Err(GeozeroError::Coord);
                }
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
                "Unknown geometry type {geometry_type:?}"
            )))
        }
    }
    Ok(())
}

fn read_coordinate<P: GeomProcessor>(
    processor: &mut P,
    geometry: &Geometry,
    offset: usize,
    idx: usize,
) -> Result<()> {
    let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
    if offset >= xy.len() / 2 {
        return Err(GeozeroError::Coord);
    }
    let dims = processor.dimensions();
    let z = if dims.z {
        dimension_value(geometry.z(), offset)?
    } else {
        None
    };
    let m = if dims.m {
        dimension_value(geometry.m(), offset)?
    } else {
        None
    };
    let t = if dims.t {
        dimension_value(geometry.t(), offset)?
    } else {
        None
    };
    let tm = if dims.tm {
        time_value(geometry.tm(), offset)?
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
    // `length` is file-controlled through `ends` (a single-part geometry passes `xy.len()`),
    // so bound the window before indexing: an odd length has no y for its last x.
    if offset % 2 != 0 || length % 2 != 0 || offset > xy.len() || length > xy.len() - offset {
        return Err(GeozeroError::Geometry(
            "FlatGeobuf geometry coordinate range exceeds the xy array".to_string(),
        ));
    }
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
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() <= 1
    {
        if let Some(xy) = geometry.xy() {
            processor.multilinestring_begin(1, idx)?;
            read_multilinestring_part(processor, geometry, 0, xy.len(), 0)?;
            processor.multilinestring_end(idx)?;
        }
    } else {
        check_ends(geometry)?;
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.multilinestring_begin(ends.len(), idx)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) as usize;
            read_multilinestring_part(processor, geometry, offset * 2, (end - offset) * 2, i)?;
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
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() <= 1
    {
        if let Some(xy) = geometry.xy() {
            processor.triangle_begin(tagged, 1, idx)?;
            read_multilinestring_part(processor, geometry, 0, xy.len(), 0)?;
            processor.triangle_end(tagged, idx)?;
        }
    } else {
        check_ends(geometry)?;
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) as usize;
            processor.triangle_begin(tagged, 1, i)?;
            read_multilinestring_part(processor, geometry, offset * 2, (end - offset) * 2, 0)?;
            offset = end;
            processor.triangle_end(tagged, i)?;
        }
    }
    Ok(())
}

fn read_tin<P: GeomProcessor>(processor: &mut P, geometry: &Geometry, idx: usize) -> Result<()> {
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() <= 1
    {
        processor.tin_begin(1, idx)?;
        read_triangle(processor, geometry, false, 0)?;
        processor.tin_end(idx)?;
    } else {
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.tin_begin(ends.len(), idx)?;
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
    if geometry.ends().is_none() || geometry.ends().ok_or(GeozeroError::GeometryFormat)?.len() <= 1
    {
        // single ring
        processor.polygon_begin(tagged, 1, idx)?;
        let xy = geometry.xy().ok_or(GeozeroError::Coord)?;
        processor.linestring_begin(false, xy.len() / 2, 0)?;
        read_coords(processor, geometry, 0, xy.len())?;
        processor.linestring_end(false, 0)?;
        processor.polygon_end(tagged, idx)?;
    } else {
        // multiple rings
        check_ends(geometry)?;
        let ends = geometry.ends().ok_or(GeozeroError::GeometryFormat)?;
        processor.polygon_begin(tagged, ends.len(), idx)?;
        let mut offset = 0;
        for i in 0..ends.len() {
            let end = ends.get(i) as usize;
            let length = (end - offset) * 2;
            processor.linestring_begin(false, length / 2, i)?;
            read_coords(processor, geometry, offset * 2, length)?;
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
                processor.linestring_begin(false, xy.len() / 2, i)?;
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
                read_polygon(processor, &geometry, true, i)?;
            }
            GeometryType::CurvePolygon if polygon_allowed => {
                read_curvepolygon(processor, &geometry, i)?;
            }
            _ => {
                return Err(GeozeroError::Geometry(format!(
                    "Unexpected geometry type in curve: {geometry_type:?}"
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

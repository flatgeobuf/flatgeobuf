use crate::feature_generated::*;
use crate::header_generated::*;
use byteorder::{ByteOrder, LittleEndian};
use geozero::error::{GeozeroError, Result};
use geozero::GeozeroGeometry;
use geozero::{ColumnValue, GeomProcessor, PropertyProcessor};
use std::str;

/// Access to the *current* feature during iteration.
pub struct FgbFeature {
    pub(crate) header_buf: Vec<u8>, // Using type Header<'a> instead of Vec would require adding a lifetime to FgbFeature
    pub(crate) feature_buf: Vec<u8>,
}

impl FgbFeature {
    pub(crate) fn header(&self) -> Header<'_> {
        // SAFETY: verification is done before creating instance
        unsafe { size_prefixed_root_as_header_unchecked(&self.header_buf) }
    }
    /// Flatbuffers feature access
    pub fn fbs_feature(&self) -> Feature<'_> {
        // SAFETY: verification is done before creating instance
        unsafe { size_prefixed_root_as_feature_unchecked(&self.feature_buf) }
    }
    /// Flatbuffers geometry access
    pub fn geometry(&self) -> Option<Geometry<'_>> {
        self.fbs_feature().geometry()
    }

    fn dimension(&self) -> geo_traits::Dimensions {
        match (self.header().has_z(), self.header().has_m()) {
            (true, true) => geo_traits::Dimensions::Xyzm,
            (true, false) => geo_traits::Dimensions::Xyz,
            (false, true) => geo_traits::Dimensions::Xym,
            (false, false) => geo_traits::Dimensions::Xy,
        }
    }

    /// Access the underlying geometry, returning an object that implements
    /// [`geo_traits::GeometryTrait`].
    ///
    /// This allows for random-access zero-copy vector data interoperability, even with Z, M, and
    /// ZM geometries that `geo_types` does not currently support.
    ///
    /// ### Notes:
    ///
    /// - Any `T` values are currently ignored.
    /// - This will error on curve geometries since they are not among the core geometry types
    ///   supported by [`geo_traits`].
    pub fn geometry_trait(
        &self,
    ) -> std::result::Result<Option<impl geo_traits::GeometryTrait<T = f64> + use<'_>>, crate::Error>
    {
        if let Some(geom) = self.geometry() {
            let dim = self.dimension();
            let result = match self.header().geometry_type() {
                GeometryType::Point => crate::geo_trait_impl::Geometry::Point(
                    crate::geo_trait_impl::Point::new(geom, dim),
                ),
                GeometryType::LineString => crate::geo_trait_impl::Geometry::LineString(
                    crate::geo_trait_impl::LineString::new(geom, dim),
                ),
                GeometryType::Polygon => crate::geo_trait_impl::Geometry::Polygon(
                    crate::geo_trait_impl::Polygon::new(geom, dim),
                ),
                GeometryType::MultiPoint => crate::geo_trait_impl::Geometry::MultiPoint(
                    crate::geo_trait_impl::MultiPoint::new(geom, dim),
                ),
                GeometryType::MultiLineString => crate::geo_trait_impl::Geometry::MultiLineString(
                    crate::geo_trait_impl::MultiLineString::new(geom, dim),
                ),
                GeometryType::MultiPolygon => crate::geo_trait_impl::Geometry::MultiPolygon(
                    crate::geo_trait_impl::MultiPolygon::new(geom, dim),
                ),
                GeometryType::Unknown => crate::geo_trait_impl::Geometry::new(geom, dim),
                GeometryType::GeometryCollection => {
                    crate::geo_trait_impl::Geometry::GeometryCollection(
                        crate::geo_trait_impl::GeometryCollection::new(geom, dim),
                    )
                }
                geom_type => {
                    return Err(crate::Error::UnsupportedGeometryType(format!(
                        "Unsupported geometry type in geo-traits: {geom_type:?}",
                    )))
                }
            };
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl geozero::FeatureAccess for FgbFeature {}

impl GeozeroGeometry for FgbFeature {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        let geometry = self
            .fbs_feature()
            .geometry()
            .ok_or(GeozeroError::GeometryFormat)?;
        let geometry_type = self.header().geometry_type();
        geometry.process(processor, geometry_type)
    }
}

impl geozero::FeatureProperties for FgbFeature {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, reader: &mut P) -> Result<bool> {
        if self.header().columns().is_none() {
            return Ok(false);
        }

        let columns_meta = self
            .header()
            .columns()
            .ok_or(GeozeroError::GeometryFormat)?;
        let mut finish = false;
        if let Some(properties) = self.fbs_feature().properties() {
            let mut offset = 0;
            let bytes = properties.bytes();
            while offset + 1 < properties.len() && !finish {
                // NOTE: it should be offset < properties.len(), but there is data with a
                // trailing byte in the last column of type Binary
                let column_idx = LittleEndian::read_u16(take(bytes, &mut offset, 2)?) as usize;
                if column_idx >= columns_meta.len() {
                    // NOTE: reading also fails if column._type is different from effective entry
                    return Err(GeozeroError::GeometryFormat);
                }
                let column = &columns_meta.get(column_idx);
                let value = match column.type_() {
                    ColumnType::Int => {
                        ColumnValue::Int(LittleEndian::read_i32(take(bytes, &mut offset, 4)?))
                    }
                    ColumnType::Long => {
                        ColumnValue::Long(LittleEndian::read_i64(take(bytes, &mut offset, 8)?))
                    }
                    ColumnType::ULong => {
                        ColumnValue::ULong(LittleEndian::read_u64(take(bytes, &mut offset, 8)?))
                    }
                    ColumnType::Double => {
                        ColumnValue::Double(LittleEndian::read_f64(take(bytes, &mut offset, 8)?))
                    }
                    // unsafe variant without UTF-8 checking would be faster...
                    ColumnType::String => ColumnValue::String(take_str(bytes, &mut offset)?),
                    ColumnType::Byte => ColumnValue::Byte(take(bytes, &mut offset, 1)?[0] as i8),
                    ColumnType::UByte => ColumnValue::UByte(take(bytes, &mut offset, 1)?[0]),
                    ColumnType::Bool => ColumnValue::Bool(take(bytes, &mut offset, 1)?[0] != 0),
                    ColumnType::Short => {
                        ColumnValue::Short(LittleEndian::read_i16(take(bytes, &mut offset, 2)?))
                    }
                    ColumnType::UShort => {
                        ColumnValue::UShort(LittleEndian::read_u16(take(bytes, &mut offset, 2)?))
                    }
                    ColumnType::UInt => {
                        ColumnValue::UInt(LittleEndian::read_u32(take(bytes, &mut offset, 4)?))
                    }
                    ColumnType::Float => {
                        ColumnValue::Float(LittleEndian::read_f32(take(bytes, &mut offset, 4)?))
                    }
                    // JSON may be represented using UTF-8, UTF-16, or UTF-32. The default encoding is UTF-8.
                    ColumnType::Json => ColumnValue::Json(take_str(bytes, &mut offset)?),
                    ColumnType::DateTime => ColumnValue::DateTime(take_str(bytes, &mut offset)?),
                    ColumnType::Binary => {
                        let len = LittleEndian::read_u32(take(bytes, &mut offset, 4)?) as usize;
                        ColumnValue::Binary(take(bytes, &mut offset, len)?)
                    }
                    ColumnType(_) => continue,
                };
                finish = reader.property(column_idx, column.name(), &value)?;
            }
        }
        Ok(finish)
    }
}

/// The next `len` bytes of a feature's properties, advancing `offset` past them.
///
/// Column indexes, values and value lengths all come from the file and are not covered by the
/// FlatBuffers verifier, so a read past the end of the properties is reported as an error
/// instead of slicing out of bounds.
fn take<'b>(bytes: &'b [u8], offset: &mut usize, len: usize) -> Result<&'b [u8]> {
    let end = offset
        .checked_add(len)
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| {
            GeozeroError::Property(
                "FlatGeobuf property value runs past the end of the feature's properties"
                    .to_string(),
            )
        })?;
    let value = &bytes[*offset..end];
    *offset = end;
    Ok(value)
}

/// A length-prefixed UTF-8 property value (String, Json, DateTime), as [`take`].
fn take_str<'b>(bytes: &'b [u8], offset: &mut usize) -> Result<&'b str> {
    let len = LittleEndian::read_u32(take(bytes, offset, 4)?) as usize;
    str::from_utf8(take(bytes, offset, len)?)
        .map_err(|_| GeozeroError::Property("Invalid UTF-8 encoding".to_string()))
}

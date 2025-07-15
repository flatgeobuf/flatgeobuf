use crate::feature_generated::*;
use crate::header_generated::*;
use byteorder::{ByteOrder, LittleEndian};
use geozero::error::{GeozeroError, Result};
use geozero::GeozeroGeometry;
use geozero::{ColumnValue, GeomProcessor, PropertyProcessor};
use std::mem::size_of;
use std::str;

/// Access to current feature
pub struct FgbFeature {
    pub(crate) header_buf: Vec<u8>, // Using type Header<'a> instead of Vec would require adding a lifetime to FgbFeature
    pub(crate) feature_buf: Vec<u8>,
}

impl FgbFeature {
    pub(crate) fn header(&self) -> Header {
        // SAFETY: verification is done before creating instance
        unsafe { size_prefixed_root_as_header_unchecked(&self.header_buf) }
    }
    /// Flatbuffers feature access
    pub fn fbs_feature(&self) -> Feature {
        // SAFETY: verification is done before creating instance
        unsafe { size_prefixed_root_as_feature_unchecked(&self.feature_buf) }
    }
    /// Flatbuffers geometry access
    pub fn geometry(&self) -> Option<Geometry> {
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
                let column_idx = LittleEndian::read_u16(&bytes[offset..offset + 2]) as usize;
                offset += size_of::<u16>();
                if column_idx >= columns_meta.len() {
                    // NOTE: reading also fails if column._type is different from effective entry
                    return Err(GeozeroError::GeometryFormat);
                }
                let column = &columns_meta.get(column_idx);
                match column.type_() {
                    ColumnType::Int => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Int(LittleEndian::read_i32(&bytes[offset..offset + 4])),
                        )?;
                        offset += size_of::<i32>();
                    }
                    ColumnType::Long => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Long(LittleEndian::read_i64(&bytes[offset..offset + 8])),
                        )?;
                        offset += size_of::<i64>();
                    }
                    ColumnType::ULong => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::ULong(LittleEndian::read_u64(&bytes[offset..offset + 8])),
                        )?;
                        offset += size_of::<u64>();
                    }
                    ColumnType::Double => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Double(LittleEndian::read_f64(
                                &bytes[offset..offset + 8],
                            )),
                        )?;
                        offset += size_of::<f64>();
                    }
                    ColumnType::String => {
                        let len = LittleEndian::read_u32(&bytes[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::String(
                                // unsafe variant without UTF-8 checking would be faster...
                                str::from_utf8(&bytes[offset..offset + len]).map_err(|_| {
                                    GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                })?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::Byte => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Byte(bytes[offset] as i8),
                        )?;
                        offset += size_of::<i8>();
                    }
                    ColumnType::UByte => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::UByte(bytes[offset]),
                        )?;
                        offset += size_of::<u8>();
                    }
                    ColumnType::Bool => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Bool(bytes[offset] != 0),
                        )?;
                        offset += size_of::<u8>();
                    }
                    ColumnType::Short => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Short(LittleEndian::read_i16(&bytes[offset..offset + 2])),
                        )?;
                        offset += size_of::<i16>();
                    }
                    ColumnType::UShort => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::UShort(LittleEndian::read_u16(
                                &bytes[offset..offset + 2],
                            )),
                        )?;
                        offset += size_of::<u16>();
                    }
                    ColumnType::UInt => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::UInt(LittleEndian::read_u32(&bytes[offset..offset + 4])),
                        )?;
                        offset += size_of::<u32>();
                    }
                    ColumnType::Float => {
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Float(LittleEndian::read_f32(&bytes[offset..offset + 4])),
                        )?;
                        offset += size_of::<f32>();
                    }
                    ColumnType::Json => {
                        let len = LittleEndian::read_u32(&bytes[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Json(
                                // JSON may be represented using UTF-8, UTF-16, or UTF-32. The default encoding is UTF-8.
                                str::from_utf8(&bytes[offset..offset + len]).map_err(|_| {
                                    GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                })?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::DateTime => {
                        let len = LittleEndian::read_u32(&bytes[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::DateTime(
                                // unsafe variant without UTF-8 checking would be faster...
                                str::from_utf8(&bytes[offset..offset + len]).map_err(|_| {
                                    GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                })?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::Binary => {
                        let len = LittleEndian::read_u32(&bytes[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            column_idx,
                            column.name(),
                            &ColumnValue::Binary(&bytes[offset..offset + len]),
                        )?;
                        offset += len;
                    }
                    ColumnType(_) => {}
                }
            }
        }
        Ok(finish)
    }
}

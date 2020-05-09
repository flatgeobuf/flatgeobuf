use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use byteorder::{ByteOrder, LittleEndian};
use geozero::error::{GeozeroError, Result};
use geozero::{ColumnValue, FeatureProcessor, PropertyProcessor};
use std::collections::HashMap;
use std::mem::size_of;
use std::str;

/// Access to current feature
pub struct FgbFeature {
    pub(crate) header_buf: Vec<u8>,
    pub(crate) feature_buf: Vec<u8>,
}

impl FgbFeature {
    pub(crate) fn header(&self) -> Header {
        get_root_as_header(&self.header_buf[..])
    }
    // Flatbuffers feature access
    pub fn fbs_feature(&self) -> Feature {
        get_root_as_feature(&self.feature_buf[..])
    }
    // Flatbuffers geometry access
    pub fn geometry(&self) -> Option<Geometry> {
        self.fbs_feature().geometry()
    }
    /// Process current feature
    pub fn process<R: FeatureProcessor>(&self, reader: &mut R, idx: u64) -> Result<()> {
        reader.feature_begin(idx)?;
        reader.properties_begin()?;
        let _ = self.process_properties(reader)?;
        reader.properties_end()?;
        reader.geometry_begin()?;
        let geometry = self
            .fbs_feature()
            .geometry()
            .ok_or(GeozeroError::GeometryFormat)?;
        let geometry_type = self.header().geometry_type();
        geometry.process(reader, geometry_type)?;
        reader.geometry_end()?;
        reader.feature_end(idx)
    }
    pub fn process_properties<R: PropertyProcessor>(&self, reader: &mut R) -> Result<bool> {
        let columns_meta = self
            .header()
            .columns()
            .ok_or(GeozeroError::GeometryFormat)?;
        let mut finish = false;
        if let Some(properties) = self.fbs_feature().properties() {
            let mut offset = 0;
            while offset < properties.len() - 1 && !finish {
                // NOTE: it should be offset < properties.len(), but there is data with a
                // trailing byte in the last column of type Binary
                let i = LittleEndian::read_u16(&properties[offset..offset + 2]) as usize;
                offset += size_of::<u16>();
                let column = &columns_meta.get(i);
                match column.type_() {
                    ColumnType::Int => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Int(LittleEndian::read_i32(
                                &properties[offset..offset + 4],
                            )),
                        )?;
                        offset += size_of::<i32>();
                    }
                    ColumnType::Long => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Long(LittleEndian::read_i64(
                                &properties[offset..offset + 8],
                            )),
                        )?;
                        offset += size_of::<i64>();
                    }
                    ColumnType::ULong => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::ULong(LittleEndian::read_u64(
                                &properties[offset..offset + 8],
                            )),
                        )?;
                        offset += size_of::<u64>();
                    }
                    ColumnType::Double => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Double(LittleEndian::read_f64(
                                &properties[offset..offset + 8],
                            )),
                        )?;
                        offset += size_of::<f64>();
                    }
                    ColumnType::String => {
                        let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::String(
                                // unsafe variant without UTF-8 checking would be faster...
                                str::from_utf8(&properties[offset..offset + len]).map_err(
                                    |_| {
                                        GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                    },
                                )?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::Byte => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Byte(properties[offset] as i8),
                        )?;
                        offset += size_of::<i8>();
                    }
                    ColumnType::UByte => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::UByte(properties[offset]),
                        )?;
                        offset += size_of::<u8>();
                    }
                    ColumnType::Bool => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Bool(properties[offset] != 0),
                        )?;
                        offset += size_of::<u8>();
                    }
                    ColumnType::Short => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Short(LittleEndian::read_i16(
                                &properties[offset..offset + 2],
                            )),
                        )?;
                        offset += size_of::<i16>();
                    }
                    ColumnType::UShort => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::UShort(LittleEndian::read_u16(
                                &properties[offset..offset + 2],
                            )),
                        )?;
                        offset += size_of::<u16>();
                    }
                    ColumnType::UInt => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::UInt(LittleEndian::read_u32(
                                &properties[offset..offset + 4],
                            )),
                        )?;
                        offset += size_of::<u32>();
                    }
                    ColumnType::Float => {
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Float(LittleEndian::read_f32(
                                &properties[offset..offset + 4],
                            )),
                        )?;
                        offset += size_of::<f32>();
                    }
                    ColumnType::Json => {
                        let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Json(
                                // JSON may be represented using UTF-8, UTF-16, or UTF-32. The default encoding is UTF-8.
                                str::from_utf8(&properties[offset..offset + len]).map_err(
                                    |_| {
                                        GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                    },
                                )?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::DateTime => {
                        let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::DateTime(
                                // unsafe variant without UTF-8 checking would be faster...
                                str::from_utf8(&properties[offset..offset + len]).map_err(
                                    |_| {
                                        GeozeroError::Property("Invalid UTF-8 encoding".to_string())
                                    },
                                )?,
                            ),
                        )?;
                        offset += len;
                    }
                    ColumnType::Binary => {
                        let len = LittleEndian::read_u32(&properties[offset..offset + 4]) as usize;
                        offset += size_of::<u32>();
                        finish = reader.property(
                            i,
                            &column.name(),
                            &ColumnValue::Binary(&properties[offset..offset + len]),
                        )?;
                        offset += len;
                    }
                }
            }
        }
        Ok(finish)
    }
    /// Return all properties in a HashMap
    /// Use `process_properties` for zero-copy access
    pub fn properties(&self) -> Result<HashMap<String, String>> {
        let mut properties = HashMap::new();
        let _ = self.process_properties(&mut properties)?;
        Ok(properties)
    }
}

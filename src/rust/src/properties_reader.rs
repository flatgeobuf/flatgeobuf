use crate::feature_generated::flat_geobuf::*;
use crate::header_generated::flat_geobuf::*;
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::mem::size_of;
use std::str;

pub struct ColumnMeta {
    pub coltype: ColumnType,
    pub name: String,
    pub index: usize,
}

fn columns_meta(header: &Header) -> Vec<ColumnMeta> {
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

impl Header<'_> {
    pub fn columns_meta(&self) -> Vec<ColumnMeta> {
        columns_meta(self)
    }
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

fn iter_properties<R>(feature: &Feature, header: &Header, mut reader: R) -> bool
where
    R: FnMut(usize, &str, ColumnValue) -> bool,
{
    let columns_meta = header.columns().unwrap();
    let mut finish = false;
    if let Some(properties) = feature.properties() {
        let mut offset = 0;
        while offset < properties.len() && !finish {
            let i = LittleEndian::read_u16(&properties[offset..offset + 2]) as usize;
            offset += size_of::<u16>();
            let column = &columns_meta.get(i);
            match column.type_() {
                ColumnType::Int => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::Int(LittleEndian::read_i32(&properties[offset..offset + 4])),
                    );
                    offset += size_of::<i32>();
                }
                ColumnType::Long => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::Long(LittleEndian::read_i64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<i64>();
                }
                ColumnType::ULong => {
                    finish = reader(
                        i,
                        &column.name(),
                        ColumnValue::ULong(LittleEndian::read_u64(&properties[offset..offset + 8])),
                    );
                    offset += size_of::<u64>();
                }
                ColumnType::Double => {
                    finish = reader(
                        i,
                        &column.name(),
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
                        &column.name(),
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

fn properties_map(feature: &Feature, header: &Header) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let _ = iter_properties(&feature, &header, |_i, colname, colval| {
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

impl Feature<'_> {
    pub fn iter_properties<R>(&self, header: &Header, reader: R) -> bool
    where
        R: FnMut(usize, &str, ColumnValue) -> bool,
    {
        iter_properties(self, &header, reader)
    }
    pub fn properties_map(&self, header: &Header) -> HashMap<String, String> {
        properties_map(self, &header)
    }
}

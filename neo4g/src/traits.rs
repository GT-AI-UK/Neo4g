use std::collections::HashMap;

use neo4rs::{BoltNull, BoltType};

use chrono::{NaiveDate, NaiveTime, NaiveDateTime};

pub enum PropValue {
    /// Covers both String and &str via From implementations.
    Str(String),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Date(NaiveDate),
    Time(NaiveTime),
    DateTime(NaiveDateTime),
}

// Allow conversion from String and &str
impl From<String> for PropValue {
    fn from(s: String) -> Self {
        PropValue::Str(s)
    }
}

impl From<&str> for PropValue {
    fn from(s: &str) -> Self {
        PropValue::Str(s.to_string())
    }
}

// Implement Into<BoltType> so that PropValue can be passed to methods
// expecting a type convertible into BoltType.
// (Here we assume that the BoltType variants have corresponding From implementations.)
impl Into<BoltType> for PropValue {
    fn into(self) -> BoltType {
        match self {
            PropValue::Str(s) => BoltType::String(s.into()),
            PropValue::I32(i) => BoltType::Integer(i.into()),
            PropValue::I64(i) => BoltType::Integer(i.into()),
            // PropValue::Date(d) => BoltType::Date(d.into()),
            // PropValue::Time(t) => BoltType::Time(t.into()),
            // PropValue::DateTime(dt) => BoltType::DateTime(dt.into()),
            _ => BoltType::Null(BoltNull),
        }
    }
}


pub trait Neo4gEntity {
    type Props;
    type ParamValue;

    fn get_entity_type(&self) -> String;
    fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, PropValue>);
    fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, PropValue>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, PropValue>);
}



pub trait Neo4gProp: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn key(&self) -> &'static str;
    fn value(&self) -> String;
}

// pub trait Neo4gEntityObjectSafe {
//     fn get_entity_type(&self) -> String;
//     fn match_by_obj(&self, props: &[Box<dyn Neo4gProp>])
//         -> (String, std::collections::HashMap<String, BoltType>);
//     fn merge_by_obj(&self, props: &[Box<dyn Neo4gProp>])
//         -> (String, std::collections::HashMap<String, BoltType>);
//     fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
// }
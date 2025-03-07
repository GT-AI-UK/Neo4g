use std::collections::HashMap;
use neo4rs::{
    Node,
    Relation,
    BoltType,
    BoltString,
    BoltBoolean,
    BoltMap,
    BoltNull,
    BoltInteger,
    BoltFloat,
    BoltList,
    BoltNode,
    BoltRelation,
    BoltUnboundedRelation,
    BoltPoint2D,
    BoltPoint3D,
    BoltBytes,
    BoltPath,
    BoltDuration,
    BoltDate,
    BoltTime,
    BoltLocalTime,
    BoltDateTime,
    BoltLocalDateTime,
    BoltDateTimeZoneId,
};
use crate::entity_wrapper::EntityWrapper;
use neo4g_derive::Neo4gNode;
use crate::traits::Neo4gEntity;

#[derive(Neo4gNode)]
pub struct UserTemplate {
    id: i32,
    name: String,
}

#[derive(Neo4gNode)]
pub struct GroupTemplate {
    id: i32,
    name: String,
    something: String,
}

// pub enum UserProps {
//     Id(i32),
//     Name(String),
// }

// impl Neo4gEntity for User {
//     type Props = UserProps;
//     fn get_entity_type(&self) -> String {
//         "Test".to_string()
//     }

//     fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
//         let mut testsdf: HashMap<String, String> = HashMap::new();
//         testsdf.insert("Test".to_string(), "Test".to_string());
//         ("Test".to_string(), testsdf)
//     }

//     fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
//         let mut testsdf: HashMap<String, String> = HashMap::new();
//         testsdf.insert("Test".to_string(), "Test".to_string());
//         ("Test".to_string(), testsdf)
//     }
// }


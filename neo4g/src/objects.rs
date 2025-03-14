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
use neo4g_derive::{Neo4gNode, not_query_param};
use crate::traits::Neo4gEntity;

//macros for neo_use? or a generic use* for a re-export of all the shit (probably better)
//take a param for default lables as well? create another prop for additional lables in structs?
#[derive(Neo4gNode)] 
pub struct UserTemplate {
    id: i32,
    name: String,
    #[not_query_param]
    groups: Vec<Group>,
}

#[derive(Neo4gNode)]
pub struct GroupTemplate {
    id: i32,
    name: String,
    something: String,
}

// impl User {
//     pub fn create_from_self(self) -> (String, Vec<(String, BoltType)>) {
//         let query = //CREATE etc.
//         let mut params: Vec<String, BoltType> = Vec::new();
//         params.push("id", self.id());
//         params.push("name", self.name());
//         (query, params)
//     }
// }

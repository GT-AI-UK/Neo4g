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
use neo4g_derive::{Neo4gNode, Neo4gRelation, not_query_param};
use crate::traits::Neo4gEntity;
use heck::ToShoutySnakeCase;
use serde::{Serialize, Deserialize};

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct GroupTemplate {
    id: i32,
    name: String,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct MemberOfTemplate {
    deleted: bool,
}

//macros for neo_use? or a generic use* for a re-export of all the shit (probably better)
//take a param for default lables as well? create another prop for additional lables in structs?
#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)] //
pub struct UserTemplate {
    id: i32,
    name: String,
    #[serde(skip)]
    password: String,
    forename: String,
    surname: String,
    deleted: bool,
    #[not_query_param]
    groups: Vec<Group>,
    #[serde(skip)]
    example: String,
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

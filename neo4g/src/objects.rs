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
use crate::query_builder::EntityType;
use crate::traits::WrappedNeo4gEntity;
use crate::entity_wrapper::{EntityWrapper, Nothing};
use neo4g_derive::{Neo4gNode, Neo4gRelation, not_query_param};
use crate::traits::Neo4gEntity;
use heck::ToShoutySnakeCase;
use serde::{Serialize, Deserialize};
use crate::traits::QueryParam;
use crate::traits::Aliasable;
use crate::query_builder::DbEntityWrapper;

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct GroupTemplate {
    id: String,
    name: String,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct MemberOfTemplate {
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct HasComponentTemplate {
    deleted: bool,
}



//macros for neo_use? or a generic use* for a re-export of all the shit (probably better)
//take a param for default lables as well? create another prop for additional lables in structs?
#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)] //
pub struct UserTemplate {
    id: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ComponentType {
    Type1,
    Type2,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Formatter::write_str(f,
            match self {
                Self::Type1 => "Type1",
                Self::Type2 => "Type2",
                _ => "",
            }
        )
    }
}

impl From<ComponentType> for BoltType {
    fn from(value: ComponentType) -> Self {
        BoltType::String(format!("{}", value).into())
    }
}

impl From<String> for ComponentType {
    fn from(value: String) -> Self {
        let v = value.to_lowercase();
        match v.as_ref() {
            "type1" => Self::Type1,
            "type2" => Self::Type2,
            _ => Self::Type1,
        }
    }
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::Type1
    }
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct ComponentTemplate {
    id: String,
    path: String,
    component_type: ComponentType,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct PageTemplate {
    id: String,
    path: String,
    #[not_query_param]
    components: Vec<Component>,
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

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
use neo4g::query_builder::EntityType;
use neo4g::traits::WrappedNeo4gEntity;
use crate::entity_wrapper::{EntityWrapper, Nothing};
use neo4g_derive::{Neo4gProp, Neo4gNode, Neo4gRelation, not_query_param};
use neo4g::traits::{Prop, Neo4gEntity};
use heck::ToShoutySnakeCase;
use serde::{Serialize, Deserialize};
use neo4g::traits::QueryParam;
use neo4g::traits::Aliasable;
use neo4g::query_builder::DbEntityWrapper;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct GroupTemplate {
    id: String,
    name: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct MemberOfTemplate {
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct HasComponentTemplate {
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)] //
pub struct UserTemplate {
    id: String,
    name: String,
    #[serde(skip)]
    password: String,
    forename: String,
    surname: String,
    #[not_query_param]
    groups: Vec<GroupTemplate>,
    #[serde(skip)]
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gProp, Serialize, Deserialize, Debug, Clone, Default)]
pub enum ComponentType {
    #[default]
    Type1,
    Type2,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct ComponentTemplate {
    id: String,
    path: String,
    component_type: ComponentType,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct PageTemplate {
    id: String,
    path: String,
    #[not_query_param]
    components: Vec<Component>,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

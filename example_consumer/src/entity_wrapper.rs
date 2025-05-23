use chrono::NaiveDateTime;
use neo4g_macro_rules::generate_entity_wrappers;
use paste::paste;
use crate::objects::{User, Group, UserProps, GroupProps, MemberOf, MemberOfProps, Page, Component, HasComponent, HasComponentProps, PageProps, ComponentProps};
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper, Neo4gLabels, Neo4gNode};
use heck::ToPascalCase;
use neo4g::traits::{Neo4gEntity, QueryParam, WrappedNeo4gEntity, Aliasable, Neo4gLabel, Paramable};
use serde::{Serialize, Deserialize};
use neo4g::query_builder::{EntityType, Array, FunctionCall, Unwinder, DbEntityWrapper};
use uuid::Uuid;
use std::collections::HashMap;

use neo4rs::{
    Node,
    Relation,
    BoltType,
    BoltString,
    BoltBoolean,
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
    BoltLocalDateTime,
};

    #[derive(Neo4gNode, Clone, Debug)]
    pub struct NothingTemplate {
        pub nothing: bool,
    }

    #[derive(Neo4gNode, Clone, Debug)]
    pub struct ValueTemplate {
        pub int: i32,
        pub float: f64,
        pub datetime: NaiveDateTime,
        pub string: String,
    }

generate_entity_wrappers!(Nothing, Value, User, Group, MemberOf, HasComponent, Page, Component);

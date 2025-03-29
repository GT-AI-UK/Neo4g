use neo4g_macro_rules::generate_entity_wrappers;
use paste::paste;
use crate::objects::{User, Group, UserProps, GroupProps, MemberOf, MemberOfProps, Page, Component, HasComponent, HasComponentProps, PageProps, ComponentProps};
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper, Neo4gLabels, Neo4gNode};
use heck::ToPascalCase;
use crate::traits::{Neo4gEntity, QueryParam};
use crate::traits::Aliasable;
use serde::{Serialize, Deserialize};
use crate::query_builder::EntityType;
use crate::query_builder::DbEntityWrapper;
use crate::traits::WrappedNeo4gEntity;

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

    #[derive(Neo4gNode, Clone, Debug)]
    pub struct NothingTemplate {
        pub nothing: bool,
    }

generate_entity_wrappers!(Nothing, User, Group, MemberOf, HasComponent, Page, Component);
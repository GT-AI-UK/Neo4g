use neo4g_macro_rules::generate_entity_wrappers;
use paste::paste;
use crate::objects::{User, Group, UserProps, GroupProps, MemberOf, MemberOfProps, Page, Component, HasComponent, HasComponentProps, PageProps, ComponentProps};
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper, Neo4gLabels, Neo4gNode};
use heck::ToPascalCase;
use neo4g::traits::{Neo4gEntity, QueryParam, WrappedNeo4gEntity, Aliasable, Neo4gLabel};
use serde::{Serialize, Deserialize};
use neo4g::query_builder::EntityType;
use neo4g::query_builder::DbEntityWrapper;

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
use neo4g_macro_rules::generate_entity_wrappers;
use paste::paste;
use crate::objects::{User, Group};
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper};
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
use neo4g_derive::Neo4gNode;
use crate::traits::Neo4gEntity;

    #[derive(Neo4gNode, Clone, Debug)]
    pub struct NothingTemplate {
        pub nothing: bool,
    }

generate_entity_wrappers!(Nothing, User, Group); // copy from_node fn into relation and vice versa to resolve (not perfect, but good enough!)
// generate props wrapper: PropsWrapper {UserProps, GroupProps, etc.} impl functions to unwrap s in entity wrapper
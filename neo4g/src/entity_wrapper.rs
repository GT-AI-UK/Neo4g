use neo4g_macro_rules::{generate_props_wrapper, generate_entity_wrapper};
use paste::paste;
use crate::objects::{User, Group};
use neo4g_derive::Neo4gEntityWrapper;
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

generate_entity_wrapper!(Nothing, User, Group);

// impl EntityWrapper {
//     pub fn from_node(node: Node) -> Self { // need to generate from macro - need if labels.contains for each variant
//         let labels = node.labels();
//         if labels.contains(&"User") {
//             println!("lables containers user");
//             return EntityWrapper::User(User::from(node));
//         }
//         let nothing = Nothing::new(true);
//         EntityWrapper::Nothing(nothing)
//     }
//     // pub fn from_relation(relation: Relation) -> Self {
//     //     let user = User::new(32, "test".to_string());
//     //     EntityWrapper::User(user)
//     // }
// }
//     pub fn from_node_struct<T: Neo4gEntity>(node: &T) -> Self {
//         EntityWrapper::Type(node)
//     }
//     pub fn from_relation_struct<T: Neo4gEntity>(relation: &T) -> Self {
//         let user = User::new(32, "test".to_string());
//         EntityWrapper::User(user)
//     }
// }
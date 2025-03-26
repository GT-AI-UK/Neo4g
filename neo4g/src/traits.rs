use std::collections::HashMap;

use neo4rs::{BoltType, Node, Relation};

use crate::{entity_wrapper::EntityWrapper, query_builder::EntityType};

pub trait Neo4gEntity {
    type Props: QueryParam;
    fn get_entity_type(&self) -> EntityType;
    fn get_label(&self) -> String;
    fn set_alias(&mut self, alias: &str) -> ();
    fn get_alias(&self) -> String;
    fn entity_by(&self, alias: &str, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
}

pub trait Neo4gNodeEntity: Neo4gEntity {
    fn from_node(node: Node) -> EntityWrapper;
}

pub trait Neo4gRelationEntity: Neo4gEntity {
    fn from_relation(relation: Relation) -> EntityWrapper;
}

pub trait QueryParam {
    fn to_query_param(&self) -> (&'static str, BoltType);
}
// pub trait Neo4gProp: std::any::Any {
//     fn as_any(&self) -> &dyn std::any::Any;
//     fn key(&self) -> &'static str;
//     fn value(&self) -> String;
// }

// pub trait Neo4gEntityObjectSafe {
//     fn get_entity_type(&self) -> String;
//     fn get_label(&self) -> String;
//     fn entity_by_obj(&self, props: &[Box<dyn Neo4gProp>])
//         -> (String, std::collections::HashMap<String, BoltType>);
//     fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
// }
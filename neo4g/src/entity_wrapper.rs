use neo4g_macro_rules::{generate_props_wrapper, generate_entity_wrapper};
use paste::paste;
use crate::objects::{User, Group};
use neo4g_derive::Neo4gEntityWrapper;
use neo4rs::{Node, Relation};
use neo4g_derive::Neo4gNode;
use crate::traits::Neo4gEntity;

#[derive(Neo4gNode, Clone, Debug)]
pub struct NothingTemplate {
    pub nothing: bool,
}

generate_entity_wrapper!(Nothing, User, Group);

impl EntityWrapper {
    pub fn from_node(self, node: Node) -> Self {
        let user = User::new(32, "test".to_string());
        EntityWrapper::User(user)
    }
    pub fn from_relation(self, relation: Relation) -> Self {
        let user = User::new(32, "test".to_string());
        EntityWrapper::User(user)
    }
}
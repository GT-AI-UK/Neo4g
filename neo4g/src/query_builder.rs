use crate::entity_wrapper::EntityWrapper;
use crate::objects::{User, Group};
use crate::traits::Neo4gEntity;

use std::collections::HashMap;

pub struct Neo4gBuilder {
    query: String,
    params: HashMap<String, String>,
    node_number: i32,
    relationship_number: i32,
    return_refs: Vec<String>,
    previous_entity: Option<String>,
}

impl Neo4gBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relationship_number: 0,
            return_refs: Vec::new(),
            previous_entity: None,
        }
    }

    pub fn create_node<T: Neo4gEntity>(mut self, entity: &T) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        self
    }

    pub fn match_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.match_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    pub fn merge_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    pub fn relate_inline<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.relationship_number += 1;
        let name = format!("neo4g_rel{}", self.relationship_number);
        self.previous_entity = Some(name.clone());
        self.query.push_str(&format!("-[neo4g_rel{}:]->", self.relationship_number));//, self.relationship_type));
        self
    }

    pub fn build(self) -> (String, HashMap<String, String>) {
        (self.query, self.params)
    }

    //async fn run(self) -> Vec<Neo4gEntity>
        //use the hashmap of return_val -("neo4j alias", returnType,eg. User)
        //query the database and return a vec of Neo4gEntities from within the EntityWrapper - database query must be declared here 
        //because that's where the match arms can be generated for the unwrapping of the Entity vec!

    //fn run? (could send query, params, and return values to neo4rs runner?)
}
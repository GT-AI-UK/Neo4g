use crate::entity_wrapper::EntityWrapper;
use crate::objects::{User, Group};
use crate::traits::Neo4gEntity;
use neo4rs::{Query, Node, Graph};

use std::collections::HashMap;

pub struct Neo4gBuilder {
    query: String,
    params: HashMap<String, String>,
    node_number: i32,
    relationship_number: i32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<String>,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Node,
    Relation,
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

    pub fn returns(mut self, alias: &[&str], entity_type: &[EntityType], entity: &[EntityWrapper]) -> Self {
        self.query.push_str("RETURN ");
        self.query.push_str(&alias.join(", "));
        self.return_refs.ex((alias.to_string(), entity_type, entity));
        self
    }

    pub fn build(self) -> (String, HashMap<String, String>) { // add returns to query string here and in run_query, or add in the return method (above)?
        (self.query, self.params)
    }

    pub async fn run_query(self, graph: Graph) -> anyhow::Result<Vec<EntityWrapper>> {
        let query = Query::new(self.query);
        let mut return_vec: Vec<EntityWrapper> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            while let Ok(Some(row)) = result.next().await {
                for (alias, entity_type, ret_obj) in self.return_refs.clone() {
                    match entity_type {
                        EntityType::Node => {
                            if let Ok(node) = row.get(&alias) {
                                return_vec.push(ret_obj.from_node(node));
                            } else {
                                println!("error getting {} from db result", alias);
                            }
                        },
                        EntityType::Relation => {
                            if let Ok(relation) = row.get(&alias) {
                                return_vec.push(ret_obj.from_relation(relation));
                            } else {
                                println!("error getting {} from db result", alias);
                            }
                        },
                    }
                }
            }
        }
        Ok(return_vec)
    }
}
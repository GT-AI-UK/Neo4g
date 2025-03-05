use neo4rs::{Graph, Query, Node, Relation, BoltType};
use std::collections::HashMap;

pub trait Neo4gEntityTrait {
    type Props;
    fn entity_from_node(node: neo4rs::Node) -> Neo4gEntity;
    fn create_node_from<T: Neo4gEntity>(node: T) -> (String, HashMap<String, BoltType>);
    fn get_name() -> String;
    // fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
    // fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
}

pub enum EntityType {
    Node,
    Relation,
}

pub struct Neo4gEntity {
    entity_type: EntityType,
    name: String,
    props: HashMap<String, BoltType>,
}

pub struct Neo4gBuilder {
    query: String,
    params: HashMap<String, String>,
    node_number: i32,
    relationship_number: i32,
    return_refs: Vec<String>,
    previous_entity: Option<String>,
}

impl Neo4gBuilder {
    fn new() -> Self {
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
        let entity_name = entity.get_name();
        let name = format!("{}{}", entity_name, self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.create_node_from(entity);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    pub async fn run_query<T>(graph: Graph, query: Query, returns: Vec<(&str, EntityType, T)>) -> anyhow::Result<Vec<Neo4gEntity>> where T: Neo4gEntityTrait {
        let return_vec: Vec<Neo4gEntity> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            while let Ok(Some(row)) = result.next().await {
                for (alias, entity_type, ret_obj) in returns {
                    match entity_type {
                        EntityType::Node => {
                            if let Ok(entity) = row.get(alias) {
                                return_vec.push(ret_obj.entity_from_node(entity));
                            } else {
                                println!("error getting {} from db result", alias);
                            }
                        },
                        EntityType::Relation => {
                            if let Ok(entity) = row.get(alias) {
                                return_vec.push(ret_obj.entity_from_node(entity));
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

    
    // pub fn match_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
    //     self.node_number += 1;
    //     let name = format!("neo4g_node{}", self.node_number);
    //     self.previous_entity = Some(name.clone());
    //     let (query_part, params) = entity.match_by(props);
    //     self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
    //     self.params.extend(params);
    //     self
    // }

    // pub fn merge_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
    //     self.node_number += 1;
    //     let name = format!("neo4g_node{}", self.node_number);
    //     self.previous_entity = Some(name.clone());
    //     let (query_part, params) = entity.merge_by(props);
    //     self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
    //     self.params.extend(params);
    //     self
    // }

    // pub fn relate_inline<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
    //     self.relationship_number += 1;
    //     let name = format!("neo4g_rel{}", self.relationship_number);
    //     self.previous_entity = Some(name.clone());
    //     self.query.push_str(&format!("-[neo4g_rel{}:]->", self.relationship_number));//, self.relationship_type));
    //     self
    // }

    // pub fn build(self) -> (String, HashMap<String, String>) {
    //     (self.query, self.params)
    // }
}
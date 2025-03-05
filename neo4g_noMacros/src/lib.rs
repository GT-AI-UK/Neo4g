use neo4rs::{Graph, Query, Node, Relation, BoltType};
use std::collections::HashMap;

pub trait Neo4gEntityTrait {
    fn entity_from_node(node: neo4rs::Node) -> Neo4gEntity;
    fn create_node_from<T: Neo4gEntityTrait>(node: T) -> (String, HashMap<String, BoltType>);
    fn get_name() -> String;
    // fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
    // fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
}

pub trait Neo4gPropsTrait {

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
    graph: Graph,
    query: String,
    params: HashMap<String, String>,
    node_number: i32,
    relationship_number: i32,
    return_refs: Vec<(String, EntityType, dyn Neo4gEntityTrait)>,
    previous_entity: Option<String>,
}

impl Neo4gBuilder {
    pub fn new(graph: Graph) -> Self {
        Self {
            graph,
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relationship_number: 0,
            return_refs: Vec::new(),
            previous_entity: None,
        }
    }

    pub fn create_node<T: Neo4gEntityTrait>(mut self, entity: &T) -> Self {
        self.node_number += 1;
        let entity_name = entity.get_name();
        let name = format!("{}{}", entity_name, self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.create_node_from(entity);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    // pub fn add_to_return<T: Neo4gEntityTrait>(mut self, alias: &str, entity_type: EntityType, entity: T) -> Self {
    //     self.return_refs.push((alias.to_string(), entity_type, entity));
    //     self
    // }

    // pub async fn run_query(self) -> anyhow::Result<Vec<Neo4gEntity>> {
    //     let return_vec: Vec<Neo4gEntity> = Vec::new();
    //     if let Ok(mut result) = self.graph.execute(self.query).await {
    //         while let Ok(Some(row)) = result.next().await {
    //             for (alias, entity_type, ret_obj) in self.return_refs {
    //                 match entity_type {
    //                     EntityType::Node => {
    //                         if let Ok(entity) = row.get(alias) {
    //                             return_vec.push(ret_obj.entity_from_node(entity));
    //                         } else {
    //                             println!("error getting {} from db result", alias);
    //                         }
    //                     },
    //                     EntityType::Relation => {
    //                         if let Ok(entity) = row.get(alias) {
    //                             return_vec.push(ret_obj.entity_from_node(entity));
    //                         } else {
    //                             println!("error getting {} from db result", alias);
    //                         }
    //                     },
    //                 }
    //             }
    //         }
    //     }
    //     Ok(return_vec)
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

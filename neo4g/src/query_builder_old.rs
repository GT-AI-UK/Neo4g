use crate::entity_wrapper::EntityWrapper;
use crate::objects::{User, Group};
use crate::traits::Neo4gEntity;
use neo4rs::{BoltNull, BoltType, Graph, Node, Relation, Query};
use std::marker::PhantomData;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Neo4gBuilder<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relationship_number: u32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause,

    _state: PhantomData<State>,
}

impl<S> Neo4gBuilder<S> {
    /// Consumes self and returns a new builder with the marker type changed to NewState.
    pub fn transition<NewState>(self) -> Neo4gBuilder<NewState> {
        let Neo4gBuilder {
            query,
            params,
            node_number,
            relationship_number,
            return_refs,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gBuilder {
            query,
            params,
            node_number,
            relationship_number,
            return_refs,
            previous_entity,
            clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl Neo4gBuilder<Empty> {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relationship_number: 0,
            return_refs: Vec::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
}

impl Neo4gBuilder<Called> {
    pub fn inner(start_num:u32) -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: start_num,
            relationship_number: start_num,
            return_refs: Vec::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
}

impl<Q: CanCreate> Neo4gBuilder<Q> {
    pub fn create_node<T: Neo4gEntity>(mut self, entity: T) -> Neo4gBuilder<CreatedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.clause = Clause::Create;
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedNode>()
    }

    pub fn merge_node<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gBuilder<CreatedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.clause = Clause::Merge;
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedNode>()
    }

    // pub fn delete_node<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props], detach: bool) -> Neo4gBuilder<DeletedEntity>
    // where EntityWrapper: From<T>, T: Clone {
    //     self.clause = Clause::Delete;
    //     let (query_part, params) = entity.delete_by(props);
    //     let (query_part, params) = ("//Not Implemented".to_string(), vec![("//Not Implemented".to_string(),BoltType::Null(BoltNull))]);
    //     self.query.push_str(&query_part);
    //     self.params.extend(params);
    //     self.transition::<DeletedEntity>()
    // }
}

impl<Q: CanMatch> Neo4gBuilder<Q> {
    pub fn match_node<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gBuilder<MatchedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.clause = Clause::Match;
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.match_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<MatchedNode>()
    }
}

// impl<Q: CanInlineRelate> Neo4gBuilder<Q> {
//     pub fn relate_inline<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gBuilder<CreatedInlineRelation>
//     where EntityWrapper: From<T>, T: Clone {
//         // IMPLEMENT THESE!
//         self.relationship_number += 1;
//         let name = format!("neo4g_rel{}", self.relationship_number);
//         self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
//         let (query_part, params) = entity.inline_with(props);
//         self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
//         self.params.extend(params);
//         //self.query.push_str(&format!("-[neo4g_rel{}:]->", self.relationship_number));//, self.relationship_type));
//         self.transition::<CreatedInlineRelation>()
//     }
// }

// impl<Q: CanInlineNode> Neo4gBuilder<Q> {
//     pub fn node_inline<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gBuilder<CreatedNode>
//     where EntityWrapper: From<T>, T: Clone {
//         self.node_number += 1;
//         let name = format!("neo4g_rel{}", self.relationship_number);
//         self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
//         let (query_part, params) = entity.inline_with(props);
//         self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
//         self.params.extend(params);
//         self.transition::<CreatedNode>()
//     }
// }

// impl<Q: CanInlineNode> Neo4gBuilder<Q> {
//     pub fn node_inline_match<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gBuilder<MatchedNode>
//     where EntityWrapper: From<T>, T: Clone {
//         self.node_number += 1;
//         let name = format!("neo4g_rel{}", self.relationship_number);
//         self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
//         let (query_part, params) = entity.inline_match_with(props);
//         self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
//         self.params.extend(params);
//         self.transition::<MatchedNode>()
//     }
// }

impl<Q: CanCall> Neo4gBuilder<Q> {
    pub fn call(mut self, inner_bulder: Neo4gBuilder<Called>) -> Neo4gBuilder<Called> {
        let (query, params) = inner_bulder.build();
        self.node_number += 100;
        self.relationship_number += 100;
        self.query.push_str(format!("CALL {{\n {} \n}}\n", &query).as_str());
        self.params.extend(params);
        self.transition::<Called>()
    }
}

impl<Q: CanAddReturn> Neo4gBuilder<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some(previous) = self.previous_entity.clone() {
            self.return_refs.push(previous);
        }
        self
    }
}

impl<Q: CanReturn> Neo4gBuilder<Q> {
    // pub fn relate_between<T: Neo4gEntity>(mut self, source: &str, dest: &str, entity: T, props: &[T::Props]) -> Neo4gBuilder<CreatedRelation>
    // where EntityWrapper: From<T>, T: Clone {
    //     // IMPLEMENT THESE!
    //     self.relationship_number += 1;
    //     let name = format!("neo4g_rel{}", self.relationship_number);
    //     self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
    //     let (query_part, params) = entity.relate_with(source, dest, props);
    //     self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
    //     self.params.extend(params);
    //     self.transition::<CreatedRelation>()
    // }

    pub fn set_returns(mut self, returns: &[(String, EntityType, EntityWrapper)]) -> Neo4gBuilder<ReturnSet> {
        if returns.is_empty() && self.return_refs.is_empty() {
            println!("Nothing will be returned from this query...");
        }
        if !returns.is_empty() {
            self.return_refs = returns.to_owned();
        }
        if !self.return_refs.is_empty() {
            self.query.push_str("RETURN ");
            let aliases: Vec<String> = self.return_refs.iter().map(|(alias, _, _)| alias.clone()).collect();
            self.query.push_str(&aliases.join(", "));
        }
        self.transition::<ReturnSet>()
    }

    pub fn build(self) -> (String, HashMap<String, BoltType>) { // add returns to query string here and in run_query, or add in the return method (above)?
        (self.query, self.params)
    }
}

impl<T: CanRun> Neo4gBuilder<T> {
    pub async fn run_query(self, graph: Graph) -> anyhow::Result<Vec<EntityWrapper>> {
        println!("query: {}", self.query.clone());
        let query = Query::new(self.query).params(self.params);
        let mut return_vec: Vec<EntityWrapper> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            println!("query ran");
            while let Ok(Some(row)) = result.next().await {
                for (alias, entity_type, ret_obj) in self.return_refs.clone() {
                    match entity_type {
                        EntityType::Node => {
                            if let Ok(node) = row.get::<Node>(&alias) {
                                println!("got node for: {}", &alias);
                                // let labels = node.labels();
                                // println!("got labels: {:?}", labels.clone());
                                let wrapped_entity = EntityWrapper::from_node(node.clone());
                                return_vec.push(wrapped_entity);
                            } else {
                                println!("error getting {} from db result", alias);
                            }
                        },
                        EntityType::Relation => {
                            // if let Ok(relation) = row.get::<Relation>(&alias) {
                            //     println!("got relation for: {}", &alias);
                            //     let label = relation.typ();
                            //     println!("got labels: {:?}", label.clone()); //probably not label?
                            //     let wrapped_entity = EntityWrapper::from_relation(relation.clone());
                            //     return_vec.push(wrapped_entity);
                            // } else {
                            //     println!("error getting {} from db result", alias);
                            // }
                        },
                    }
                }
            }
        }
        Ok(return_vec)
    }
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Node,
    Relation,
}

#[derive(Clone, Debug)]
pub enum Clause {
    Create,
    Merge,
    Match,
    Delete,
    None,
}

pub trait CanMatch {}
pub trait CanCreate {}
pub trait CanInlineNode {}
pub trait CanInlineRelate {}
pub trait CanCall {}
pub trait CanReturn {}
pub trait CanRun {}

pub trait CanAddReturn {}

#[derive(Debug, Clone)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct CreatedNode;

#[derive(Debug, Clone)]
pub struct MatchedNode;

#[derive(Debug, Clone)]
pub struct CreatedInlineRelation;

#[derive(Debug, Clone)]
pub struct CreatedRelation;

#[derive(Debug, Clone)]
pub struct InLineRelation;

#[derive(Debug, Clone)]
pub struct ReturnSet;

#[derive(Debug, Clone)]
pub struct DeletedEntity;

#[derive(Debug, Clone)]
pub struct Called;

impl CanMatch for Empty {}
impl CanCreate for Empty {}
impl CanCall for Empty {}
impl CanInlineRelate for CreatedNode {}
impl CanReturn for CreatedNode {}
impl CanInlineRelate for MatchedNode {}
impl CanMatch for MatchedNode {}
impl CanCreate for MatchedNode {}
impl CanCall for MatchedNode {}
impl CanReturn for MatchedNode {}
impl CanInlineNode for CreatedInlineRelation {}
impl CanReturn for CreatedRelation {}
impl CanRun for ReturnSet {}
impl CanRun for DeletedEntity {}
impl CanReturn for Called {}
impl CanRun for Called {}
impl CanAddReturn for CreatedNode {}
impl CanAddReturn for MatchedNode {}
impl CanAddReturn for CreatedInlineRelation {}
impl CanAddReturn for CreatedRelation {}
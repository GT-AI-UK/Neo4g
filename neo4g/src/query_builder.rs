use crate::entity_wrapper::EntityWrapper;
use crate::objects::{User, Group};
use crate::traits::Neo4gEntity;
use neo4rs::{BoltNull, BoltType, Graph, Node, Relation, Query};
use std::marker::PhantomData;

// Could start Neo4gBuilder again but use statements as functions too?
// eg.
// Neo4gBuilder::new().create().node(entity).relation(enitty2).node(entity3).ret(e1, e2, e3).run().await?
// Neo4gBuilder::new().get().node(entity).relation(en2).node(en3).where(props).ret(e1, e3).run().await? //get instead of match
// Neo4gBuilder::new().merge(None).node(e1).zero_plus().relation(e2).node(e3).on_match_set(props).on_create_set(props).with(e1, e3).merge_ref(e1).relation(e4).node(e5).run().await?
// // unsure whether to have merge take params... can I create a hashmap in the query builder for which nodes are which aliases? Can I validate aliases or are they better as &str?
// instead of using merge with optional tuple, could have a .props() method? .merge().node(node).props(props)?
// should additional_labels be a special type of field, or just exlude it from macros and assume it exists? C_ould probably create the field in the macro as a pub Vec<String>
// .where may need to be named differently? similar to match...
// using ref of previous node/rel would be interesting too, methods with _ref appended? conditions to call these would be that the statement hasn't just started, so would need yet more states for builder to be in?
// If doing this, need to have differernt structs to navigate between for each different clause?
// Structs/Traits to be in the form: <Clause><PreviousAction>, eg. MergeReferencedNode, MergeReferencedRelation, MatchRefNode, MatchRefRelation

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Neo4gBuilder<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause, // use clause to determine what .node and .relation call. permissions for where will be interesting. 
    _state: PhantomData<State>,
}

impl Neo4gBuilder<Empty> {
    // Constructors
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relation_number: 0,
            return_refs: Vec::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
    pub fn new_inner(start_num:u32) -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: start_num,
            relation_number: start_num,
            return_refs: Vec::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
    pub fn build(self) -> (String, HashMap<String, BoltType>) { // add returns to query string here and in run_query, or add in the return method (above)?
        (self.query, self.params)
    }

    // Query statements
    pub fn create(mut self) -> Neo4gCreateStatement<Empty> {
        self.clause = Clause::Create;
        self.query.push_str("CREATE ");
        Neo4gCreateStatement::from(self)
    }
    pub fn merge(mut self) -> Neo4gMergeStatement<Empty> {
        self.clause = Clause::Merge;
        self.query.push_str("MERGE ");
        Neo4gMergeStatement::from(self)
    }
    pub fn r#match(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        self.query.push_str("MATCH ");
        Neo4gMatchStatement::from(self)
    }
    pub fn optional_match(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        self.query.push_str("OPTIONAL MATCH ");
        Neo4gMatchStatement::from(self)
    }
}

//Create statement methods
impl<Q: CanNode> Neo4gCreateStatement<Q> {
    pub fn node<T: Neo4gEntity>(mut self, entity: T) -> Neo4gCreateStatement<CreatedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gCreateStatement<CreatedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gCreateStatement<CreatedNode> {
    pub fn relation<T: Neo4gEntity>(mut self, entity: T) -> Neo4gCreateStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part.replace("neo4g_relation", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_ref(mut self, relation_ref: &str) -> Neo4gCreateStatement<CreatedRelation> {
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<CreatedRelation>()
    }
    pub fn set_additional_labels(mut self, labels: &[&str]) -> Self {
        self.query = self.query.replace(":AdditionalLabels", &labels.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gCreateStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some((mut name, entity_type, entity)) = self.previous_entity.clone() {
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type, entity));
        }
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gCreateStatement<Q> {
    pub fn end_statement(mut self) -> Neo4gBuilder<CreatedNode> {
        self.query = self.query.replace(":AdditionalLabels", "");
        Neo4gBuilder::from(self)
    }
}

//Merge statement methods
impl<Q: CanNode> Neo4gMergeStatement<Q> {
    // pub fn node<T: Neo4gEntity>(mut self, entity: T) -> Neo4gMergeStatement<CreatedNode>
    // where EntityWrapper: From<T>, T: Clone {
    //     self.node_number += 1;
    //     let name = format!("neo4g_node{}", self.node_number);
    //     self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
    //     let (query_part, params) = entity.merge_from_self();
    //     self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
    //     self.params.extend(params);
    //     self.transition::<CreatedNode>()
    // }
    pub fn node<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gMergeStatement<CreatedNode> // could split this into .node and .props() using wrappers?
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gMergeStatement<CreatedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gMergeStatement<CreatedNode> {
    // pub fn relation<T: Neo4gEntity>(mut self, entity: T) -> Neo4gMergeStatement<CreatedRelation>
    // where EntityWrapper: From<T>, T: Clone {
    //     self.relation_number += 1;
    //     let label = entity.get_label();
    //     let name = format!("{}{}:AdditionalLabels", label, self.node_number);
    //     self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
    //     let (query_part, params) = entity.merge_from_self();
    //     self.query.push_str(&query_part.replace("neo4g_relation", &name.clone()));
    //     self.params.extend(params);
    //     self.transition::<CreatedRelation>()
    // }
    pub fn relation<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gMergeStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_relation", &name.clone()));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_ref(mut self, relation_ref: &str) -> Neo4gMergeStatement<CreatedRelation> {
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<CreatedRelation>()
    }
    pub fn set_additional_labels(mut self, labels: &[&str]) -> Self {
        self.query = self.query.replace(":AdditionalLabels", &labels.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gMergeStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some((mut name, entity_type, entity)) = self.previous_entity.clone() {
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type, entity));
        }
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gMergeStatement<Q> {
    pub fn on_create_set<T: Neo4gEntity>(mut self, alias: &str, entity: T, props: &[T::Props]) -> Self
    where EntityWrapper: From<T>, T: Clone {
        if self.on_create_str.is_empty() {
            self.on_create_str.push_str("ON CREATE SET\n");
        }
        //get params in the right format and join them to str
        self
    }
    pub fn on_match_set<T: Neo4gEntity>(mut self, alias: &str, entity: T, props: &[T::Props]) -> Self
    where EntityWrapper: From<T>, T: Clone {
        if self.on_match_str.is_empty() {
            self.on_match_str.push_str("ON MATCH SET\n");
        }
        //get params in the right format and join them to str
        self
    }
    pub fn end_statement(mut self) -> Neo4gBuilder<CreatedNode> {
        self.query = self.query.replace(":AdditionalLabels", "");
        println!("INSIDE MERGE! Query: {}", &self.query);
        self.query.push_str(&format!("{}\n{}", self.on_match_str, self.on_create_str));
        Neo4gBuilder::from(self)
    }
}

//Match statement methods
impl<Q: CanNode> Neo4gMatchStatement<Q> {
    // pub fn node<T: Neo4gEntity>(mut self, entity: T) -> Neo4gMatchStatement<MatchedNode>
    // where EntityWrapper: From<T>, T: Clone {
    //     self.node_number += 1;
    //     let name = format!("neo4g_node{}", self.node_number);
    //     self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
    //     let (query_part, params) = entity.match_from_self();
    //     self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
    //     self.params.extend(params);
    //     self.transition::<MatchedNode>()
    // }
    pub fn node<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gMatchStatement<MatchedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, where_str, params) = entity.match_by(props);
        if self.where_str.is_empty() {
            self.where_str.push_str("WHERE ")
        }
        self.where_str.push_str(&where_str);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self.transition::<MatchedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gMatchStatement<MatchedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<MatchedNode>()
    }
}
impl Neo4gMatchStatement<MatchedNode> {
    // pub fn relation<T: Neo4gEntity>(mut self, entity: T) -> Neo4gMatchStatement<MatchedRelation>
    // where EntityWrapper: From<T>, T: Clone {
    //     self.relation_number += 1;
    //     let label = entity.get_label();
    //     let name = format!("{}{}:AdditionalLabels", label, self.node_number);
    //     self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
    //     let (query_part, params) = entity.match_from_self();
    //     self.query.push_str(&query_part.replace("neo4g_relation", &name.clone()));
    //     self.params.extend(params);
    //     self.transition::<MatchedRelation>()
    // }
    pub fn relation<T: Neo4gEntity>(mut self, entity: T, props: &[T::Props]) -> Neo4gMatchStatement<MatchedRelation>
    where EntityWrapper: From<T>, T: Clone { // do I need this - will it generate an inner where?
        self.relation_number += 1;
        let label = entity.get_label();
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, where_str, params) = entity.match_by(props);
        if self.where_str.is_empty() {
            self.where_str.push_str("WHERE ")
        }
        self.where_str.push_str(&where_str);
        self.query.push_str(&query_part.replace("neo4g_relation", &name.clone()));
        self.params.extend(params);
        self.transition::<MatchedRelation>()
    }
    pub fn relation_ref(mut self, relation_ref: &str) -> Neo4gMatchStatement<MatchedRelation> {
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<MatchedRelation>()
    }
    pub fn set_additional_labels(mut self, labels: &[&str]) -> Self {
        self.query = self.query.replace(":AdditionalLabels", &labels.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gMatchStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some((mut name, entity_type, entity)) = self.previous_entity.clone() {
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type, entity));
        }
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gMatchStatement<Q> {
    pub fn r#where<T: Neo4gEntity>(mut self, alias: &str, entity: T, props: &[T::Props]) -> Self
    where EntityWrapper: From<T>, T: Clone {
        if self.where_str.is_empty() {
            self.where_str.push_str("WHERE ")
        }
        //get params structured correctly and join into where_str
        self
    }
    pub fn end_statement(mut self) -> Neo4gBuilder<MatchedNode> {
        self.query.push_str(&format!("{}", self.where_str));
        self.query = self.query.replace(":AdditionalLabels", "");
        Neo4gBuilder::from(self)
    }
}

//Statement combiners
impl <Q: CanAddReturn> Neo4gBuilder<Q> {
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
    pub fn with(mut self, aliases: &[&str]) -> Self {
        self.query.push_str(&format!("WITH {}\n", aliases.join(", ")));
        self
    }
    pub fn call(mut self, inner_bulder: Neo4gBuilder<Empty>) -> Self {
        let (query, params) = inner_bulder.build();
        self.node_number += 100;
        self.relation_number += 100;
        self.query.push_str(format!("CALL {{\n {} \n}}\n", &query).as_str());
        self.params.extend(params);
        self
    }
}

//Run query
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
                            if let Ok(relation) = row.get::<Relation>(&alias) {
                                println!("got relation for: {}", &alias);
                                let label = relation.typ();
                                let wrapped_entity = EntityWrapper::from_relation(relation.clone());
                                return_vec.push(wrapped_entity);
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
pub trait CanNode {}
pub trait PossibleStatementEnd {}
pub trait CanRun {}
pub trait CanAddReturn {}
pub trait CanDelete {}

#[derive(Debug, Clone)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct Statement;

#[derive(Debug, Clone)]
pub struct CreatedNode;

#[derive(Debug, Clone)]
pub struct MatchedNode;

#[derive(Debug, Clone)]
pub struct CreatedRelation;

#[derive(Debug, Clone)]
pub struct MatchedRelation;

#[derive(Debug, Clone)]
pub struct ReturnSet;

#[derive(Debug, Clone)]
pub struct Called;

#[derive(Debug, Clone)]
pub struct DeletedEntity;

impl CanMatch for Empty {}
impl CanCreate for Empty {}
impl CanDelete for MatchedNode {}
impl CanRun for ReturnSet {}
impl CanRun for DeletedEntity {}

impl PossibleStatementEnd for MatchedNode {}
impl PossibleStatementEnd for CreatedNode {}
impl PossibleStatementEnd for ReturnSet {}
impl CanMatch for MatchedNode {}
impl CanCreate for MatchedNode {}
impl CanNode for CreatedRelation {}
impl CanNode for Empty {}
impl CanNode for MatchedRelation {}

impl CanAddReturn for CreatedNode {}
impl CanAddReturn for MatchedNode {}
impl CanAddReturn for CreatedRelation {}
impl CanAddReturn for MatchedRelation {}


#[derive(Debug, Clone)]
pub struct Neo4gMatchStatement<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    where_str: String,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause, // use clause to determine what .node and .relation call. permissions for where will be interesting. 
    _state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct Neo4gMergeStatement<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    on_create_str: String,
    on_match_str: String,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause, // use clause to determine what .node and .relation call. permissions for where will be interesting. 
    _state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct Neo4gCreateStatement<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause, // use clause to determine what .node and .relation call. permissions for where will be interesting. 
    _state: PhantomData<State>,
}

impl<S> Neo4gBuilder<S> {
    /// Consumes self and returns a new builder with the marker type changed to NewState.
    fn transition<NewState>(self) -> Neo4gBuilder<NewState> {
        let Neo4gBuilder {
            query,
            params,
            node_number,
            relation_number,
            return_refs,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gBuilder {
            query,
            params,
            node_number,
            relation_number,
            return_refs,
            previous_entity,
            clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl<S> Neo4gMatchStatement<S> {
    /// Consumes self and returns a new builder with the marker type changed to NewState.
    fn transition<NewState>(self) -> Neo4gMatchStatement<NewState> {
        let Neo4gMatchStatement {
            query,
            params,
            node_number,
            relation_number,
            where_str,
            return_refs,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gMatchStatement {
            query,
            params,
            node_number,
            relation_number,
            where_str,
            return_refs,
            previous_entity,
            clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl<S> Neo4gMergeStatement<S> {
    /// Consumes self and returns a new builder with the marker type changed to NewState.
    fn transition<NewState>(self) -> Neo4gMergeStatement<NewState> {
        let Neo4gMergeStatement {
            query,
            params,
            node_number,
            relation_number,
            on_create_str,
            on_match_str,
            return_refs,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gMergeStatement {
            query,
            params,
            node_number,
            relation_number,
            on_create_str,
            on_match_str,
            return_refs,
            previous_entity,
            clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl<S> Neo4gCreateStatement<S> {
    /// Consumes self and returns a new builder with the marker type changed to NewState.
    fn transition<NewState>(self) -> Neo4gCreateStatement<NewState> {
        let Neo4gCreateStatement {
            query,
            params,
            node_number,
            relation_number,
            return_refs,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gCreateStatement {
            query,
            params,
            node_number,
            relation_number,
            return_refs,
            previous_entity,
            clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gBuilder<S>> for Neo4gCreateStatement<Empty> {
    fn from(value: Neo4gBuilder<S>) -> Neo4gCreateStatement<Empty> {
        Neo4gCreateStatement::<Empty> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gBuilder<S>> for Neo4gMergeStatement<Empty> {
    fn from(value: Neo4gBuilder<S>) -> Neo4gMergeStatement<Empty> {
        Neo4gMergeStatement::<Empty> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            on_create_str: "".to_string(),
            on_match_str: "".to_string(),
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gBuilder<S>> for Neo4gMatchStatement<Empty> {
    fn from(value: Neo4gBuilder<S>) -> Neo4gMatchStatement<Empty> {
        Neo4gMatchStatement::<Empty> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            where_str: "".to_string(),
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gMatchStatement<S>> for Neo4gBuilder<MatchedNode> {
    fn from(value: Neo4gMatchStatement<S>) -> Neo4gBuilder<MatchedNode> {
        Neo4gBuilder::<MatchedNode> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gMergeStatement<S>> for Neo4gBuilder<CreatedNode> {
    fn from(value: Neo4gMergeStatement<S>) -> Neo4gBuilder<CreatedNode> {
        Neo4gBuilder::<CreatedNode> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

impl <S> From<Neo4gCreateStatement<S>> for Neo4gBuilder<CreatedNode> {
    fn from(value: Neo4gCreateStatement<S>) -> Neo4gBuilder<CreatedNode> {
        Neo4gBuilder::<CreatedNode> {
            query: value.query,
            params: value.params,
            node_number: value.node_number,
            relation_number: value.relation_number,
            return_refs: value.return_refs,
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}
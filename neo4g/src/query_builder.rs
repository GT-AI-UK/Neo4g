use crate::entity_wrapper::{EntityWrapper, PropsWrapper};
use crate::traits::{Neo4gEntity, QueryParam};
use neo4rs::{BoltNull, BoltType, Graph, Node, Relation, Query};
use std::marker::PhantomData;
use std::fmt;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Neo4gBuilder<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    unwind_number: u32,
    set_number: u32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    order_by_str: String,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause,
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
            unwind_number: 0,
            set_number: 0,
            return_refs: Vec::new(),
            order_by_str: String::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
    pub fn new_inner(node_number:u32, relation_number: u32, unwind_number: u32, set_number: u32) -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number,
            relation_number,
            unwind_number,
            set_number,
            return_refs: Vec::new(),
            order_by_str: String::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
    pub fn build(self) -> (String, HashMap<String, BoltType>) {
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
    pub fn get(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        self.query.push_str("MATCH ");
        Neo4gMatchStatement::from(self)
    }
    pub fn optional_match(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        self.query.push_str("OPTIONAL MATCH ");
        Neo4gMatchStatement::from(self)
    }
    //where to put unwind?
}

impl<Q: CanWith> Neo4gBuilder<Q> {
    pub fn unwind(mut self, mut unwinder: Unwinder) -> Self {
        self.unwind_number += 1;
        unwinder.alias = format!("neo4g_unwind{}", self.unwind_number);
        let (query, params) = unwinder.unwind();
        self.query.push_str(&format!("{}\n", query));
        self.params.extend(params);
        self
    }
    pub fn with(mut self, entities_to_alias: &[&EntityWrapper]) -> Neo4gBuilder<Withed> {
        let aliases: Vec<String> = entities_to_alias.iter().map(|entity| {
            entity.get_alias()
        }).collect();
        self.query.push_str(&format!("WITH {}\n", aliases.join(", ")));
        self.transition::<Withed>()
    }
    //pub fn with_parameterised_array(mut self, param: ParamString) need a way to alias automatically?
}

impl<Q: CanWhere> Neo4gBuilder<Q> {
    pub fn filter_with(mut self, filter: Where<Condition>) -> Self { // needs to be specific to with... I'd rather not have lots of filters on it...
        if self.where_str.is_empty() {
            self.where_str.push_str("\nWHERE ")
        }
        let (query_part, where_params) = filter.build();
        self.where_str.push_str(&format!("{}\n", &query_part));
        self.params.extend(where_params);
        self
    }
    pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: T, props: &[PropsWrapper]) -> Self {
        let alias = entity_to_alias.get_alias();
        let (query, params) = PropsWrapper::set_by(&alias, self.set_number, props);
        self.params.extend(params);
        if self.set_str.is_empty() {
            self.set_str = "\nSET ".to_string();
        }
        self.set_str.push_str(&query);
        self
    }
}

//Create statement methods
impl<Q: CanNode> Neo4gCreateStatement<Q> {
    /// This is a docstring
    pub fn node<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gCreateStatement<CreatedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.node_number));
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(new_params);
        self.transition::<CreatedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gCreateStatement<CreatedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gCreateStatement<CreatedNode> {
    pub fn relation<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gCreateStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_rel", &name.clone()));
        self.params.extend(new_params);
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
    pub fn node<T: Neo4gEntity>(mut self, entity: &mut T, props: &[T::Props]) -> Neo4gMergeStatement<CreatedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.node_number));
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        if props.is_empty() {
            self.query.push_str(&format!("({})", name));
        } else {
            let (query_part, params) = entity.entity_by(props);
            let new_params = prepend_params_key(&entity.get_alias(), params);
            self.query.push_str(&query_part.replace("neo4g_node", &name));
            self.params.extend(new_params);
        }
        self.transition::<CreatedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gMergeStatement<CreatedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gMergeStatement<CreatedNode> {
    pub fn relations<T: Neo4gEntity>(mut self, min_hops: u32, entity: &mut T, props: &[T::Props]) -> Neo4gMergeStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(props);
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_rel", &name.clone()).replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(new_params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation<T: Neo4gEntity>(mut self, entity: &mut T, props: &[T::Props]) -> Neo4gMergeStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(props);
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_rel", &name.clone()).replace("*min_hops..", ""));
        self.params.extend(new_params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_flipped<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gMergeStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("-[", "<-[").replace("neo4g_rel", &name).replace("]->", "]-"));
        self.params.extend(new_params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_undirected(mut self) -> Neo4gMergeStatement<CreatedRelation> {
        self.query.push_str("--");
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
    pub fn on_create(mut self) -> Self {
        self.current_on_str = OnString::Create;
        if self.on_create_str.is_empty() {
            self.on_create_str.push_str("\nON CREATE\n");
        }
        self
    }
    pub fn on_match(mut self) -> Self {
        self.current_on_str = OnString::Match;
        if self.on_match_str.is_empty() {
            self.on_match_str.push_str("\nON MATCH\n");
        }
        self
    }
    pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: T, props: &[PropsWrapper]) -> Self {
        let alias = entity_to_alias.get_alias();
        let (query, params) = PropsWrapper::set_by(&alias, self.set_number, props);
        self.params.extend(params);
        match self.current_on_str {
            OnString::Create => {
                if self.on_create_str == "\nON CREATE\n".to_string() {
                    self.on_create_str.push_str("SET ");
                }
                self.on_create_str.push_str(&query)
            },
            OnString::Match => {
                if self.on_match_str == "\nON MATCH\n".to_string() {
                    self.on_match_str.push_str("SET ");
                }
                self.on_match_str.push_str(&query)
            },
            OnString::None => (),
        }
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
    pub fn node<T: Neo4gEntity>(mut self, entity: &mut T, props: &[T::Props]) -> Neo4gMatchStatement<MatchedNode>
    where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.node_number));
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
        if props.is_empty() {
            self.query.push_str(&format!("({})", name));
        } else {
            let (query_part, params) = entity.entity_by(props);
            let new_params = prepend_params_key(&entity.get_alias(), params);
            self.query.push_str(&query_part.replace("neo4g_node", &name));
            self.params.extend(new_params);
        }
        self.transition::<MatchedNode>()
    }
    pub fn node_ref(mut self, node_ref: &str) -> Neo4gMatchStatement<MatchedNode> {
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<MatchedNode>()
    }
}
impl Neo4gMatchStatement<MatchedNode> {
    pub fn relations<T: Neo4gEntity>(mut self, min_hops: u32, entity: &mut T, props: &[T::Props]) -> Neo4gMatchStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(props);
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_rel", &name.clone()).replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(new_params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation<T: Neo4gEntity>(mut self, entity: &mut T, props: &[T::Props]) -> Neo4gMatchStatement<MatchedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(props);
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("neo4g_rel", &name.clone()).replace("*min_hops..", ""));
        self.params.extend(new_params);
        self.transition::<MatchedRelation>()
    }
    pub fn relation_flipped<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gMatchStatement<CreatedRelation>
    where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        let new_params = prepend_params_key(&entity.get_alias(), params);
        self.query.push_str(&query_part.replace("-[", "<-[").replace("neo4g_rel", &name).replace("]->", "]-"));
        self.params.extend(new_params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_undirected(mut self) -> Neo4gMatchStatement<CreatedRelation> {
        self.query.push_str("--");
        self.transition::<CreatedRelation>()
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
    pub fn filter(mut self, filter: Where<Condition>) -> Self {
        if self.where_str.is_empty() {
            self.where_str.push_str("\nWHERE ")
        }
        let (query_part, where_params) = filter.build();
        self.where_str.push_str(&format!("{}\n", &query_part));
        self.params.extend(where_params);
        self
    }
    pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: T, props: &[PropsWrapper]) -> Self {
        let alias = entity_to_alias.get_alias();
        let (query, params) = PropsWrapper::set_by(&alias, self.set_number, props);
        self.params.extend(params);
        if self.set_str.is_empty() {
            self.set_str = "\nSET ".to_string();
        }
        self.set_str.push_str(&query);
        self
    }
    pub fn end_statement(mut self) -> Neo4gBuilder<MatchedNode> {
        if !self.where_str.is_empty() {
            self.query.push_str(&format!("{}", self.where_str));
        }
        if !self.set_str.is_empty() {
            self.query.push_str(&format!("{}", self.set_str));
            if !self.return_refs.is_empty() {
                let return_aliases: Vec<String> = self.return_refs.iter().map(|item| {
                    item.0.clone()
                }).collect();
                self.query.push_str(&format!("WITH {}\n", return_aliases.join(", ")));
            }
        }
        
        self.query = self.query.replace(":AdditionalLabels", "");
        Neo4gBuilder::from(self)
    }
}

//Statement combiners
impl <Q: PossibleQueryEnd> Neo4gBuilder<Q> {
    pub fn set_returns(mut self, returns: &[(String, EntityType, EntityWrapper)]) -> Self {
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
        self
    }

    /// inner_builder should be created with Neo4gBuilder::new_inner() in all cases I can think of.
    pub fn call(mut self, aliases: &[&str], inner_bulder: Neo4gBuilder<Empty>) -> Neo4gBuilder<Called> {
        self.node_number = inner_bulder.node_number;
        self.relation_number = inner_bulder.relation_number;
        let (query, params) = inner_bulder.build();
        self.query.push_str(format!("CALL ({}) {{\n {} \n}}\n", &aliases.join(", "), &query).as_str());
        self.params.extend(params);
        self.transition::<Called>()
    }

    pub fn skip(mut self, skip: u32) -> Self {
        self.query.push_str(&format!("SKIP {}\n", skip));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.query.push_str(&format!("LIMIT {}\n", limit));
        self
    }

    pub fn order_by<T: Neo4gEntity>(mut self, entity_to_alias: &mut T, prop: PropsWrapper, order: Order) -> Self {
        if self.order_by_str.is_empty() {
            self.order_by_str = "\nORDER BY ".to_string();
        }
        let (name, _) = prop.to_query_param();
        let alias = entity_to_alias.get_alias();
        self.order_by_str.push_str(&format!("{}.{} {}", alias, &name, order.to_string()));
        self
    }

    pub async fn run_query(mut self, graph: Graph) -> anyhow::Result<Vec<EntityWrapper>> {
        if !self.return_refs.is_empty() {
            self.query.push_str("RETURN ");
            let aliases: Vec<String> = self.return_refs.iter().map(|(alias, _, _)| alias.clone()).collect();
            self.query.push_str(&aliases.join(", "));
        }
        self.query.push_str(&self.order_by_str);
        println!("query: {}", self.query.clone());
        println!("params: {:?}", self.params.clone());
        let query = Query::new(self.query).params(self.params);
        let mut return_vec: Vec<EntityWrapper> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            println!("query ran");
            while let Ok(Some(row)) = result.next().await {
                for (alias, entity_type, ret_obj) in self.return_refs.clone() {
                    println!("attemping to get {} from database. {:?}, {:?}", alias, &entity_type, &ret_obj);
                    match entity_type {
                        EntityType::Node => {
                            if let Ok(node) = row.get::<Node>(&alias) {
                                println!("got node for: {}", &alias);
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
                                println!("wrapped relation: {:?}", wrapped_entity);
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
pub trait CanWith {}
pub trait PossibleQueryEnd {}
pub trait CanAddReturn {}
pub trait CanDelete {}
pub trait CanWhere {}

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

#[derive(Debug, Clone)]
pub struct Withed;

impl CanMatch for Empty {}
impl CanCreate for Empty {}
impl CanDelete for MatchedNode {}
impl CanMatch for Withed {}
impl CanCreate for Withed {}
impl CanDelete for Withed {}
impl CanAddReturn for Withed {}
impl CanWhere for Withed {}

impl CanWith for MatchedNode {}
impl CanWith for CreatedNode {}
impl CanWith for ReturnSet {}
impl CanWith for Called {}
impl CanWith for Empty {}

impl PossibleStatementEnd for MatchedNode {}
impl PossibleStatementEnd for CreatedNode {}
impl PossibleStatementEnd for ReturnSet {}

impl PossibleQueryEnd for MatchedNode {}
impl PossibleQueryEnd for CreatedNode {}
impl PossibleQueryEnd for Withed {}
impl PossibleQueryEnd for Called {}

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
    unwind_number: u32,
    set_number: u32,
    where_str: String,
    set_str: String,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause,
    _state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct Neo4gMergeStatement<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    unwind_number: u32,
    set_number: u32,
    on_create_str: String,
    on_match_str: String,
    current_on_str: OnString,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause,
    _state: PhantomData<State>,
}

#[derive(Debug, Clone)]
pub struct Neo4gCreateStatement<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    unwind_number: u32,
    set_number: u32,
    return_refs: Vec<(String, EntityType, EntityWrapper)>,
    previous_entity: Option<(String, EntityType, EntityWrapper)>,
    clause: Clause,
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
            unwind_number,
            set_number,
            return_refs,
            order_by_str,
            previous_entity,
            clause,
            ..
        } = self;
        Neo4gBuilder {
            query,
            params,
            node_number,
            relation_number,
            unwind_number,
            set_number,
            return_refs,
            order_by_str,
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
            unwind_number,
            set_number,
            where_str,
            set_str,
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
            unwind_number,
            set_number,
            where_str,
            set_str,
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
            unwind_number,
            set_number,
            on_create_str,
            on_match_str,
            current_on_str,
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
            unwind_number,
            set_number,
            on_create_str,
            on_match_str,
            current_on_str,
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
            unwind_number,
            set_number,
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
            unwind_number,
            set_number,
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
            on_create_str: "".to_string(),
            on_match_str: "".to_string(),
            current_on_str: OnString::None,
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
            where_str: String::new(),
            set_str: String::new(),
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
            return_refs: value.return_refs,
            order_by_str: String::new(),
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
            return_refs: value.return_refs,
            order_by_str: String::new(),
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
            unwind_number: value.unwind_number,
            set_number: value.set_number,
            return_refs: value.return_refs,
            order_by_str: String::new(),
            previous_entity: value.previous_entity,
            clause: value.clause,
            _state: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
    None,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Order::Asc => "",
            Order::Desc  => "DESC",
            Order::None => "",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for Order {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "asc" => Order::Asc,
            "desc"  => Order::Desc,
            "" => Order::None,
            _ => panic!("Invalid CompareJoiner string: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OnString {
    Create,
    Match,
    None,
}

#[derive(Debug, Clone)]
pub enum CompareOperator {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
    Ne,
    In,
}

impl fmt::Display for CompareOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CompareOperator::Eq => "=",
            CompareOperator::Gt => ">",
            CompareOperator::Ge => ">=",
            CompareOperator::Lt => "<",
            CompareOperator::Le => "<=",
            CompareOperator::Ne => "<>",
            CompareOperator::In => "IN",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for CompareOperator {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "eq" => CompareOperator::Eq,
            "gt" => CompareOperator::Gt,
            "ge" => CompareOperator::Ge,
            "lt" => CompareOperator::Lt,
            "le" => CompareOperator::Le,
            "ne" => CompareOperator::Ne,
            "in" => CompareOperator::In,
            _ => panic!("Invalid CompareOperator string: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompareJoiner {
    And,
    Or,
    Not,
}

impl fmt::Display for CompareJoiner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CompareJoiner::And => "AND",
            CompareJoiner::Or  => "OR",
            CompareJoiner::Not => "NOT",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for CompareJoiner {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "and" => CompareJoiner::And,
            "or"  => CompareJoiner::Or,
            "not" => CompareJoiner::Not,
            _ => panic!("Invalid CompareJoiner string: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Where<State> {
    string: String,
    params: HashMap<String, BoltType>,
    condition_number: u32,
    _state: PhantomData<State>,
}

pub trait CanCondition {}
pub trait CanJoin {}
pub trait CanBuild {}

#[derive(Debug, Clone)]
pub struct Condition;

#[derive(Debug, Clone)]
pub struct Joined;

impl CanCondition for Empty {}
impl CanJoin for Condition {}
impl CanBuild for Condition {}
impl CanCondition for Joined {}

impl<S> Where<S> {
    fn transition<NewState>(self) -> Where<NewState> {
        let Where {string, params, condition_number, ..} = self;
        Where {string, params, condition_number, _state: std::marker::PhantomData,}
    }
}

impl Where<Empty> {
    pub fn new() -> Self {
        Where {
            string: String::new(),
            params: HashMap::new(),
            condition_number: 0,
            _state: PhantomData,
        }
    }
}

impl<Q: CanCondition> Where<Q> {
    pub fn condition<T: Neo4gEntity>(mut self, entity_to_alias: &T, prop: PropsWrapper, operator: CompareOperator) -> Where<Condition> {
        self.condition_number += 1;
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity_to_alias.get_alias();
        self.string.push_str(&format!("{}.{} {} ${}", alias, name, operator.to_string(), &param_name));
        self.params.insert(param_name, value);
        self.transition::<Condition>()
    }
    pub fn coalesce<T: Neo4gEntity>(mut self, entity_to_alias: &T, prop: PropsWrapper) -> Where<Condition> {
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity_to_alias.get_alias();
        self.string.push_str(&format!("{}.{} = coalesce(${}, {}.{}", alias, name, param_name, alias, name));
        self.params.insert(param_name, value);
        self.transition::<Condition>()
    }
    // DO YOU EVEN NEED COALESCE?!
    // just provide the params you need by starting the query, then branching it based on what optional values you have? - if querying by ID or Username, just have .node inside and if/else?
    // should functions be done like this? I could wrap each function in an enum. Might be simpler/lower effort to template out with a macro and a FunctionWrapper?
    // seems reasonable, but as they are all so different, the implementations to get them into query, queryparams structure will be complex...
    // pub fn condition_with_function(mut self, alias: &str, prop: PropsWrapper, operator: CompareOperator, function: Function) -> Where<Condition> {
    //     let (name, value) = prop.to_query_param();
    //     let (func_query_part, (func_prop, func_val)) = function.to_query_part();
    //     self.string.push_str(&format!("{}.{} {} {}", alias, name, operator.to_string(), func_query_part));
    //     self.transition::<Condition>()
    // }
    pub fn nest(mut self, inner_builder: Where<Condition>) -> Where<Condition> {
        let (query, params) = inner_builder.build();
        self.string.push_str(&format!("({})", query));
        self.params.extend(params);
        self.transition::<Condition>()
    }
}

impl<Q: CanJoin> Where<Q> {
    pub fn join(mut self, joiner: CompareJoiner) -> Where<Joined> {
        self.string.push_str(&format!(" {} ", joiner.to_string()));
        self.transition::<Joined>()
    }
}

impl Where<Condition> {
    pub fn build(self) -> (String, HashMap<String, BoltType>) {
        (self.string, self.params)
    }
}

fn bolt_inner_value(bolt: &BoltType) -> String {
    match bolt {
        BoltType::String(s) => s.value.clone(),
        BoltType::Boolean(b) => b.value.to_string(),
        BoltType::Integer(i) => i.value.to_string(),
        BoltType::Float(f) => f.value.to_string(),
        //BoltType::LocalDateTime(d) => d ????
        // Add match arms for all variants you care about...
        _ => format!("{:?}", bolt),
    }
}

fn prepend_params_key(prefix: &str, params: HashMap<String, BoltType>) -> HashMap<String, BoltType> {
    let mut new_params: HashMap<String, BoltType> = HashMap::new();
    for (key, value) in params {
        let new_key = format!("{}_{}", prefix, key);
        new_params.insert(new_key, value);
    }
    new_params
}

#[derive(Debug, Clone)]
pub struct ParamString(String);

impl ParamString {
    pub fn new<T: Neo4gEntity>(entity: T, prop: T::Props) -> Self {
        let (key, _) = prop.to_query_param();
        Self(format!("${}_{}", entity.get_alias(), key))
    }
    pub fn manual(name: &str) -> Self {
        Self(String::from(name))
    }
    pub fn get(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Unwinder {
    alias: String,
    list: Vec<PropsWrapper>
}

impl Unwinder {
    pub fn new(list: Vec<PropsWrapper>) -> Self {
        Self {
            alias: String::new(),
            list,
        }
    }
    pub fn unwind(self) -> (String, HashMap<String, BoltType>) {
        let bolt_vec: Vec<BoltType> = self.list.iter().map(|props_wrapper| {
            let (_, bolt) = props_wrapper.to_query_param();
            bolt
        }).collect();
        let mut params = HashMap::new();
        params.insert(format!("{}", self.alias), bolt_vec.into());
        let query = format!("UNWIND ${} as {}", self.alias, self.alias);
        (query, params)
    }
}

impl Default for Unwinder {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl Neo4gEntity for Unwinder {
    type Props = UnwinderProps;
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
        (String::new(), HashMap::new())
    }
    fn get_entity_type(&self) -> String {
        String::from("unwinder")
    }
    fn get_label(&self) -> String {
        String::new()
    }
    fn entity_by(&self, _: &[Self::Props]) -> (String, HashMap<String, BoltType>) {
        (String::new(), HashMap::new())
    }
    fn set_alias(&mut self, alias: &str) {
        self.alias = alias.to_string();
    }
    fn get_alias(&self) -> String {
        self.alias.clone()
    }
}

pub enum UnwinderProps {
    RequiredToSatisfyTrait,
    OtherwiseNotUsed,
}

impl QueryParam for UnwinderProps {
    fn to_query_param(&self) -> (&'static str, BoltType) {
        ("", BoltType::Null(BoltNull))
    }
}


// REALLY NEED TO THINK ABOUT HOW TO CALL FUNCTIONS!!!
// // pub struct Avg {
// //     alias: Option<String>,
// //     props: Option<Vec<PropsWrapper>>,
// // }

// // impl Avg {
// //     pub fn new(alias: &str, props: &[PropsWrapper]) -> Self {
// //         Self {
// //             alias: Some(alias.into()),
// //             props: Some(props.into()),
// //         }
// //     }
// //     fn to_query_part(self) -> (String, (String, BoltType)) {
// //         ("Example".to_string(), HashMap::new())
// //     }
// // }

// // pub struct Count {
// //     alias: Option<String>,
// //     props: Option<Vec<PropsWrapper>>,
// // }

// // impl Count {
// //     pub fn new(alias: &str, props: &[PropsWrapper]) -> Self {
// //         Self {
// //             alias: Some(alias.into()),
// //             props: Some(props.into()),
// //         }
// //     }
// //     fn to_query_part(self) -> (String, (String, BoltType)) {
// //         ("Example".to_string(), HashMap::new())
// //     }
// // }

// // pub struct Max {
// //     alias: Option<String>,
// //     props: Option<Vec<PropsWrapper>>,
// // }

// // impl Max {
// //     pub fn new(alias: &str, props: &[PropsWrapper]) -> Self {
// //         Self {
// //             alias: Some(alias.into()),
// //             props: Some(props.into()),
// //         }
// //     }
// //     fn to_query_part(self) -> (String, (String, BoltType)) {
// //         ("Example".to_string(), HashMap::new())
// //     }
// // }

// // pub struct Min {
// //     alias: Option<String>,
// //     props: Option<Vec<PropsWrapper>>,
// // }

// // impl Min {
// //     pub fn new(alias: &str, props: &[PropsWrapper]) -> Self {
// //         Self {
// //             alias: Some(alias.into()),
// //             props: Some(props.into()),
// //         }
// //     }
// //     fn to_query_part(self) -> (String, (String, BoltType)) {
// //         ("Example".to_string(), HashMap::new())
// //     }
// // }

// // pub struct Sum {
// //     alias: Option<String>,
// //     props: Option<Vec<PropsWrapper>>,
// // }

// // impl Sum {
// //     pub fn new(alias: &str, props: &[PropsWrapper]) -> Self {
// //         Self {
// //             alias: Some(alias.into()),
// //             props: Some(props.into()),
// //         }
// //     }
// //     fn to_query_part(self) -> (String, (String, BoltType)) {
// //         ("Example".to_string(), HashMap::new())
// //     }
// // }

// pub enum Function {
//     Coalesce(PropsWrapper),
//     // Avg(Avg),
//     // Count(Count),
//     // Max(Max),
//     // Min(Min),
//     // Sum(Sum),
// }


// // impl From<Avg> for Function {
// //     fn from(value: Avg) -> Self {
// //         Self::Avg(value)
// //     }
// // }

// // impl From<Count> for Function {
// //     fn from(value: Count) -> Self {
// //         Self::Count(value)
// //     }
// // }

// // impl From<Max> for Function {
// //     fn from(value: Max) -> Self {
// //         Self::Max(value)
// //     }
// // }

// // impl From<Min> for Function {
// //     fn from(value: Min) -> Self {
// //         Self::Min(value)
// //     }
// // }

// // impl From<Sum> for Function {
// //     fn from(value: Sum) -> Self {
// //         Self::Sum(value)
// //     }
// // }

// impl Function {
//     ///Returns a query part and a params HashMap. Params is not supported within Where predicates.
//     pub fn to_query_part(self) -> (String, (String, BoltType)) {
//         match self {
//             Function::Coalesce(prop) => {
//                 let (name, _) = prop.to_query_param();
//                 let param_name = format!("coalesce_{}", name);
//                 let return_string = format!("coalesce(${}, {}.{})", &param_name, name);
//                 (return_string, (param_name, BoltType::Null(BoltNull)))
//             },
//             // Function::Avg(obj) => {
//             //     obj.to_query_part()
//             // },
//             // Function::Count(obj) => {
//             //     obj.to_query_part()
//             // },
//             // Function::Max(obj) => {
//             //     obj.to_query_part()
//             // },
//             // Function::Min(obj) => {
//             //     obj.to_query_part()
//             // },
//             // Function::Sum(obj) => {
//             //     obj.to_query_part()
//             // },
//         }
//     }
// }
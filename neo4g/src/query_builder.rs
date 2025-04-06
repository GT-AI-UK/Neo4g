use anyhow::{anyhow, Error};
use neo4rs::{query, BoltNull, BoltType, Graph, Node, Query, Relation};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use std::hash::Hash;
use std::marker::PhantomData;
use std::fmt;
use crate::traits::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Neo4gBuilder<State> {
    query: String,
    params: HashMap<String, BoltType>,
    node_number: u32,
    relation_number: u32,
    unwind_number: u32,
    set_number: u32,
    return_refs: Vec<(String, EntityType)>,
    order_by_str: String,
    previous_entity: Option<(String, EntityType)>,
    clause: Clause,
    _state: PhantomData<State>,
}

impl Neo4gBuilder<Empty> {
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
    fn new_with_parent<S>(parent: &Neo4gBuilder<S>) -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: parent.node_number,
            relation_number: parent.relation_number,
            unwind_number: parent.unwind_number,
            set_number: parent.set_number,
            return_refs: Vec::new(),
            order_by_str: String::new(),
            previous_entity: None,
            clause: Clause::None,
            _state: PhantomData,
        }
    }
}

impl<Q: CanCreate> Neo4gBuilder<Q> {
    /// Generates a CREATE statement. 
    /// # Example
    /// ```rust
    /// .create()
    ///     .node(&mut node1).add_to_return()
    ///     .relation(&mut rel).add_to_return()
    ///     .node(&mut node2).add_to_return()
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// CREATE (node1alias:Node1Label {node1_prop1: $node1_prop1, etc})-[relalias:REL_TYPE {etc}]->(node2alias: Node2Label {etc})
    /// RETURN node1alias, relalias, node2alias
    /// ```
    /// and asociated params. etc is used to save room instead of typing out loads of example props.
    /// each non-excluded property of the provided struct is used when creating the database entities.
    pub fn create(mut self) -> Neo4gCreateStatement<Empty> {
        self.clause = Clause::Create;
        if !self.query.is_empty() {
            self.query.push_str("\n");
        }
        self.query.push_str("CREATE ");
        Neo4gCreateStatement::from(self)
    }
    /// Generates a MERGE statement. 
    /// # Example
    /// ```rust
    /// .merge()
    ///     .node(&mut node1, props!(node1 => node1.prop1)).add_to_return()
    ///     .relation(&mut rel, no_props!()).add_to_return()
    ///     .node(&mut node2, props!(node2 => node2.prop1)).add_to_return()
    ///     .on_create()
    ///         .set(node1, props!(node1 => Node1Props::Prop2(val), node1.prop3)))
    ///         .set(node2, props!(node2 => node2.prop2, Node2Props::Prop3(val)))
    ///     .on_match()
    ///         .set(node1, &[&Node1Props::Eg(321)]))
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// MERGE (node1alias:Node1Label {prop: $node1_prop1)-[relalias:REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// ON CREATE SET node1alias.prop2 = $set_prop21, node2alias.prop3 = $set_prop32
    /// ON MATCH SET node1alias.prop2 = $set_prop23
    /// RETURN node1alias, relalias, node2alias
    /// ```
    /// and asociated params.
    pub fn merge(mut self) -> Neo4gMergeStatement<Empty> {
        self.clause = Clause::Merge;
        if !self.query.is_empty() {
            self.query.push_str("\n");
        }
        self.query.push_str("MERGE ");
        Neo4gMergeStatement::from(self)
    }
}

impl<Q: CanMatch> Neo4gBuilder<Q> {
    /// Generates a CALL call
    /// # Example
    /// ```rust
    /// .call(wrap!(prev1), |inner| {
    ///     inner.get()
    ///         .node(&mut entity, props!(entity => entity.prop1))
    ///         .set(&entity, props!(entity => entity.prop2, EntityProps::Prop3(val)))
    ///         .set(&prev1, props!(prev1 => prev1.prop1, PrevProps::Prop2(val)))
    ///     .end_statement()
    /// })
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// CALL (prev1alias) {
    ///     MATCH (entityalias: EntityLabel {entity.prop1: $entity_prop1})
    ///     SET entity.prop2 = $set_prop21, entity.prop3 = $set_prop32, prev1alias.prop1 = $set_prop13, prev1alias = $set_prop24
    /// }
    /// ```
    /// and asociated params for the inner builder.
    /// NOTE: You can't return anything from within a CALL block. This is a limitation of Neo4j.
    pub fn call<A, F, B>(mut self, entities_to_alias: &[&A], inner_builder_closure: F) -> Neo4gBuilder<Called>
    //pub fn call<F, B>(mut self, inner_builder_closure: F) -> Neo4gBuilder<Called>
    where A: Aliasable, F: FnOnce(Neo4gBuilder<Empty>) -> Neo4gBuilder<B>, B: PossibleQueryEnd {
        let inner_builder = Neo4gBuilder::new_with_parent(&self);
        let (
            query,
            params,
            node_number,
            relation_number,
            set_number,
            unwind_number,
            return_refs,
        ) = inner_builder_closure(inner_builder).build_inner();
        self.node_number = node_number;
        self.relation_number = relation_number;
        self.set_number = set_number;
        self.unwind_number = unwind_number;
        self.return_refs.extend_from_slice(&return_refs);
        let aliases: Vec<String> = entities_to_alias.iter().map(|entity| {
            entity.get_alias()
        }).collect();
        self.query.push_str(format!("\nCALL ({}) {{\n {} \n}}", aliases.join(", "), &query).as_str());
        //self.query.push_str(format!("\nCALL {{\n{}\n}}", &query).as_str());
        self.params.extend(params);
        self.transition::<Called>()
    }
       /// Generates an UNWIND call. 
    /// # Example
    /// ```rust
    /// .unwind(
    ///     &mut Unwinder::new(&Array::new("example_array", vec!["string".into(), 1.into()]))
    /// )
    /// .unwind(
    ///     &mut Unwinder::new(&example_array)
    /// )
    /// .unwind(&mut unwinder)
    /// ```
    /// The examples above may each generate the following query:
    /// ```rust
    /// UNWIND $example_array as unwound_example_array1
    /// ```
    /// and asociated params.
    pub fn unwind(mut self, unwinder: &mut Unwinder) -> Self {
        self.unwind_number += 1;
        unwinder.alias = format!("unwound_{}{}", &unwinder.array.alias, self.unwind_number);
        let (query, params) = unwinder.unwind();
        self.query.push_str(&query);
        self.params.extend(params);
        self
    }
    /// Generates a MATCH statement. 
    /// # Example
    /// ```rust
    /// .get()
    ///     .node(&mut node1, props!(node1 => node1.prop1)).add_to_return()
    ///     .relation(&mut rel, no_props!()).add_to_return()
    ///     .node(&mut node2, props!(node2 => Node2Props::Prop1(val))).add_to_return()
    ///     .filter(Where::new()
    ///         .condition(&node1, prop!(node1.prop2), CompareOperator::Gt)
    ///     )
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// MATCH (node1alias: Node1Label {prop: $node1_prop1})-[relalias: REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// WHERE node1alias.prop2 > $where_prop21
    /// RETURN node1alias, relalias, node2alias
    /// ```
    /// and asociated params.
    pub fn get(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        if !self.query.is_empty() {
            self.query.push_str("\n");
        }
        self.query.push_str("MATCH ");
        Neo4gMatchStatement::from(self)
    }
    /// Generates an OPTION MATCH statement. 
    /// # Example
    /// ```rust
    /// .optional_match()
    ///     .node(&mut node1, props!(node1 => node1.prop1)).add_to_return()
    ///     .relation(&mut rel, no_props!()).add_to_return()
    ///     .node(&mut node2, props!(node2 => Node2Props::Prop1(val))).add_to_return()
    ///     .filter(Where::new()
    ///         .condition(&node1, prop!(node1.prop2), CompareOperator::Gt)
    ///     )
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// OPTION MATCH (node1alias: Node1Label {prop: $node1_prop1})-[relalias: REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// WHERE node1alias.prop2 > $where_prop21
    /// RETURN node1alias, relalias, node2alias
    /// ```
    /// and asociated params.
    pub fn optional_match(mut self) -> Neo4gMatchStatement<Empty> {
        self.clause = Clause::Match;
        if !self.query.is_empty() {
            self.query.push_str("\n");
        }
        self.query.push_str("OPTIONAL MATCH ");
        Neo4gMatchStatement::from(self)
    }
}

impl<Q: CanWith> Neo4gBuilder<Q> {
    /// Generates a WITH call. 
    /// # Examples
    /// ```rust
    /// .with(&[entity1.wrap(), entity2.wrap()])
    /// .with(wrap!(entity1, entity2))
    /// ```
    /// The examples above each generate the following query:
    /// ```rust
    /// WITH entity1alias, entity2alias
    /// ```
    /// and asociated params.
    pub fn with<W: CanBuild>(mut self, with: With<W>) -> Neo4gBuilder<Withed> {
        let (query, params) = with.build();
        self.query.push_str(&format!("\nWITH {}", query));
        self.params.extend(params);
        self.transition::<Withed>()
    }
}

//Create statement methods
impl<Q: CanNode> Neo4gCreateStatement<Q> {
    /// Generates a node query object. 
    /// Uses all of the properties of the node object as properties of the node in the database.
    /// # Example
    /// ```rust
    /// .node(&mut node)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias:NodeLabel {prop1: $node1_prop1, prop2: $node1_prop2, propn: $node1_propn})
    /// ```
    /// and asociated params.
    pub fn node<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gCreateStatement<CreatedNode>
    { //where EntityWrapper: From<T>, T: Clone {
        self.node_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.node_number));
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part);
        self.params.extend(params);
        self.transition::<CreatedNode>()
    }
    /// Provides a node alias for use in a query string. 
    /// Uses all of the properties of the node object as properties of the node in the database.
    /// # Example
    /// ```rust
    /// .node_ref(&mut node)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias)
    /// ```
    pub fn node_ref<T: Neo4gEntity>(mut self, node_to_alias: &T) -> Neo4gCreateStatement<CreatedNode> {
        let node_ref = node_to_alias.get_alias();
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gCreateStatement<CreatedNode> {
    /// Generates a relation query object. 
    /// Uses all of the properties of the relation object as properties of the relation in the database.
    /// # Example
    /// ```rust
    /// .relation(&mut relation)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2, propn: $relation1_propn}]->
    /// ```
    /// and asociated params.
    pub fn relation<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gCreateStatement<CreatedRelation>
    { //where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part);
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Provides a relation alias for use in a query string. 
    /// Uses all of the properties of the relation object as properties of the relation in the database.
    /// # Example
    /// ```rust
    /// .relation(&mut relation)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias]->
    /// ```
    pub fn relation_ref<T: Neo4gEntity>(mut self, rel_to_alias: &T) -> Neo4gCreateStatement<CreatedRelation> {
        let relation_ref = rel_to_alias.get_alias();
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<CreatedRelation>()
    }
    /// Appends Labels to the node object that was created before calling this. 
    /// This can only be called once per node!
    /// Two default labels are included in the Label enum.
    /// Labels are automatically added to the enum by generate_entity_wrappers!
    /// # Example
    /// ```rust
    /// .set_additional_labels(&[Label::Any, Label::SysObj])
    /// ```
    /// The example above inserts the labels within a node object, eg. (node1:Node) becomes (node1:Node:Any:SysObj):
    pub fn set_additional_labels<T: Neo4gLabel>(mut self, labels: &[T]) -> Self {
        let additional_lables: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        self.query = self.query.replace(":AdditionalLabels", &additional_lables.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gCreateStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some((mut name, entity_type)) = self.previous_entity.clone() {
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type));
        }
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gCreateStatement<Q> {
    /// Finalises the current statement, tidies up placeholders, and changes the state of the builder so that new statements can be added.
    pub fn end_statement(mut self) -> Neo4gBuilder<CreatedNode> {
        self.query = self.query.replace(":AdditionalLabels", "");
        Neo4gBuilder::from(self)
    }
}

//Merge statement methods
impl<Q: CanNode> Neo4gMergeStatement<Q> {
    /// Generates a node query object. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .node(&mut node, props!(node => node.prop1, NodeProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias:NodeLabel {prop1: $node1_prop1, prop2: $node1_prop2})
    /// ```
    /// and asociated params.
    pub fn node<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMergeStatement<CreatedNode>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.node_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.node_number);
        entity.set_alias(&alias);
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node));
        if props.is_empty() {
            self.query.push_str(&format!("({})", name));
        } else {
            let (query_part, params) = entity.entity_by(&alias, &props);
            self.query.push_str(&query_part);
            self.params.extend(params);
        }
        self.transition::<CreatedNode>()
    }
    /// Provides a node alias for use in a query string. 
    /// Uses all of the properties of the node object as properties of the node in the database.
    /// # Example
    /// ```rust
    /// .node_ref(&mut node)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias)
    /// ```
    pub fn node_ref<T: Neo4gEntity>(mut self, node_to_alias: &T) -> Neo4gMergeStatement<CreatedNode> {
        let node_ref = node_to_alias.get_alias();
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<CreatedNode>()
    }
}
impl Neo4gMergeStatement<CreatedNode> {
    /// Generates a relation query object with a minimum number of relations traversed. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .relation(0, &mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE*0 {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relations<T, F>(mut self, min_hops: u32, entity: &mut T, props_macro: F) -> Neo4gMergeStatement<CreatedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.relation_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Generates a relation query object. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relation<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMergeStatement<CreatedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.relation_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("*min_hops..", ""));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Generates a relation query object with the arrow going right to left. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .relation_flipped(&mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// <-[realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]-
    /// ```
    /// and asociated params.
    pub fn relation_flipped<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMergeStatement<CreatedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.relation_number += 1;
        let props = props_macro(entity);
        self.relation_number += 1;
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.entity_by(&alias,&props);
        self.query.push_str(&query_part.replace("-[", "<-[").replace("]->", "]-"));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    pub fn relation_undirected(mut self) -> Neo4gMergeStatement<CreatedRelation> {
        self.query.push_str("--");
        self.transition::<CreatedRelation>()
    }
    /// Provides a relation alias for use in a query string
    /// Uses all of the properties of the relation object as properties of the relation in the database.
    /// # Example
    /// ```rust
    /// .relation(&mut relation)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias]->
    /// ```
    pub fn relation_ref<T: Neo4gEntity>(mut self, rel_to_alias: &T) -> Neo4gMergeStatement<CreatedRelation> {
        let relation_ref = rel_to_alias.get_alias();
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<CreatedRelation>()
    }
    /// Appends Labels to the node object that was created before calling this. 
    /// This can only be called once per node!
    /// Two default labels are included in the Label enum.
    /// Labels are automatically added to the enum by generate_entity_wrappers!
    /// # Example
    /// ```rust
    /// .set_additional_labels(&[Label::Any, Label::SysObj])
    /// ```
    /// The example above inserts the labels within a node object, eg. (node1:Node) becomes (node1:Node:Any:SysObj):
    pub fn set_additional_labels<T: Neo4gLabel>(mut self, labels: &[T]) -> Self {
        let additional_lables: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        self.query = self.query.replace(":AdditionalLabels", &additional_lables.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gMergeStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        if let Some((mut name, entity_type)) = self.previous_entity.clone() {
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type));
        }
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gMergeStatement<Q> {
    /// Appends ON CREATE to the query string and changes the builder state so that .set() can be called
    /// # Example
    /// ```rust
    /// .on_create()
    /// ```
    pub fn on_create(mut self) -> Self {
        self.current_on_str = OnString::Create;
        if self.on_create_str.is_empty() {
            self.on_create_str.push_str("\nON CREATE");
        }
        self
    }
    /// Appends ON MATCH to the query string and changes the builder state so that .set() can be called
    /// # Example
    /// ```rust
    /// .on_match()
    /// ```
    pub fn on_match(mut self) -> Self {
        self.current_on_str = OnString::Match;
        if self.on_match_str.is_empty() {
            self.on_match_str.push_str("\nON MATCH");
        }
        self
    }
    /// Generates a SET call
    /// # Example
    /// ```rust
    /// .set(&entity1, props!(entity1 => entity1.prop1, Entity1Props::Prop2(val)))
    /// .set(&entity2, props!(entity2 => entity2.prop1, Entity2Props::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// SET entity1alias.prop1 = $set1_prop1, entity1alias.prop2 = $set1_prop2, entity2alias.prop1 = $set2_prop1, entity2alias.prop2 = $set2_prop2
    /// ```
    /// and asociated params for the inner builder.
    pub fn set<T, F>(mut self, entity: &T, props_macro: F) -> Self
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        //where T::Props: Clone, PropsWrapper: From<<T as Neo4gEntity>::Props> {
        self.set_number += 1;
        let props = props_macro(entity);
        let alias = entity.get_alias();
        let mut query = String::new();
        let mut params = std::collections::HashMap::new();
        let props_str: Vec<String> = props
            .iter()
            .map(|prop| {
                let (key, value) = prop.to_query_param();
                params.insert(format!("set_{}{}", key.to_string(), self.set_number), value);
                format!("{}.{} = $set_{}{}\n", alias, key, key, self.set_number)
            })
            .collect();

        query.push_str(&props_str.join(", "));
        self.params.extend(params);
        match self.current_on_str {
            OnString::Create => {
                if self.on_create_str == "\nON CREATE".to_string() {
                    self.on_create_str.push_str("\nSET ");
                } else {
                    self.on_create_str.push_str(", ");
                }
                self.on_create_str.push_str(&query)
            },
            OnString::Match => {
                if self.on_match_str == "\nON MATCH".to_string() {
                    self.on_match_str.push_str("\nSET ");
                } else {
                    self.on_match_str.push_str(", ");
                }
                self.on_match_str.push_str(&query)
            },
            OnString::None => (),
        }
        self
    }
    /// Finalises the current statement, tidies up placeholders, and changes the state of the builder so that new statements can be added.
    pub fn end_statement(mut self) -> Neo4gBuilder<CreatedNode> {
        self.query = self.query.replace(":AdditionalLabels", "");
        //// println!("INSIDE MERGE! Query: {}", &self.query);
        self.query.push_str(&format!("{}{}", self.on_match_str, self.on_create_str));
        Neo4gBuilder::from(self)
    }
}

//Match statement methods
impl<Q: CanNode> Neo4gMatchStatement<Q> {
    /// Generates a node query object. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .node(&mut node, props!(node => node.prop1, NodeProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias:NodeLabel {prop1: $node1_prop1, prop2: $node1_prop2})
    /// ```
    /// and asociated params.
    pub fn node<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMatchStatement<MatchedNode>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.node_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.node_number);
        entity.set_alias(&alias);
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        self.previous_entity = Some((name.clone(), EntityType::Node));
        if props.is_empty() {
            self.query.push_str(&format!("({})", name));
        } else {
            let (query_part, params) = entity.entity_by(&alias, &props);
            self.query.push_str(&query_part);
            self.params.extend(params);
        }
        self.transition::<MatchedNode>()
    }
    /// Provides a node alias for use in a query string. 
    /// Uses all of the properties of the node object as properties of the node in the database.
    /// # Example
    /// ```rust
    /// .node_ref(&mut node)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias)
    /// ```
    pub fn node_ref<T: Neo4gEntity>(mut self, node_to_alias: &T) -> Neo4gMatchStatement<MatchedNode> {
        let node_ref = node_to_alias.get_alias();
        self.query.push_str(&format!("({})",node_ref));
        self.transition::<MatchedNode>()
    }
}
impl Neo4gMatchStatement<MatchedNode> {
    /// Generates a relation query object with a minimum number of relations traversed. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .relations(0, &mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE*0 {prop1: $relation_prop1, prop2: $relation_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relations<T, F>(mut self, min_hops: u32, entity: &mut T, props_macro: F) -> Neo4gMatchStatement<MatchedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.relation_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(params);
        self.transition::<MatchedRelation>()
    }
    /// Generates a relation query object. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE {prop1: $relation_prop1, prop2: $relation_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relation<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMatchStatement<MatchedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        // println!("incoming rel: {}", self.relation_number);
        self.relation_number += 1;
        let props: Vec<<T as Neo4gEntity>::Props> = props_macro(entity);
        let label = entity.get_label();
        // println!("rel num pre alias: {}", self.relation_number);
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        // println!("alias to be set: {}", &alias);
        entity.set_alias(&alias);
        // println!("actual alias: {}", &entity.get_alias());
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("*min_hops..", ""));
        self.params.extend(params);
        self.transition::<MatchedRelation>()
    }
    /// Generates a relation query object with the arrow going right to left. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .relation_flipped(&mut relation, props!(relation => relation.prop1, RelationProps::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// <-[realtionalias:REL_TYPE {prop1: $relation_prop1, prop2: $relation_prop2}]-
    /// ```
    /// and asociated params.
    pub fn relation_flipped<T, F>(mut self, entity: &mut T, props_macro: F) -> Neo4gMatchStatement<MatchedRelation>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.relation_number += 1;
        let props = props_macro(entity);
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        self.previous_entity = Some((name.clone(), EntityType::Relation));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part.replace("-[", "<-[").replace("]->", "]-"));
        self.params.extend(params);
        self.transition::<MatchedRelation>()
    }
    /// Provides an empty relation with no direction, simply -- . 
    pub fn relation_undirected(mut self) -> Neo4gMatchStatement<MatchedRelation> {
        self.query.push_str("--");
        self.transition::<MatchedRelation>()
    }
    /// Provides a relation alias for use in a query string
    /// Uses all of the properties of the relation object as properties of the relation in the database.
    /// # Example
    /// ```rust
    /// .relation(&mut relation)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias]->
    /// ```
    pub fn relation_ref<T: Neo4gEntity>(mut self, rel_to_alias: &T) -> Neo4gMatchStatement<MatchedRelation> {
        let relation_ref = rel_to_alias.get_alias();
        self.query.push_str(&format!("-[{}]->", relation_ref));
        self.transition::<MatchedRelation>()
    }
    /// Appends Labels to the node object that was created before calling this. 
    /// This can only be called once per node!
    /// Two default labels are included in the Label enum.
    /// Labels are automatically added to the enum by generate_entity_wrappers!
    /// # Example
    /// ```rust
    /// .set_additional_labels(&[Label::Any, Label::SysObj])
    /// ```
    /// The example above inserts the labels within a node object, eg. (node1:Node) becomes (node1:Node:Any:SysObj):
    pub fn set_additional_labels<T: Neo4gLabel>(mut self, labels: &[T]) -> Self {
        let additional_lables: Vec<String> = labels.iter().map(|l| l.to_string()).collect();
        self.query = self.query.replace(":AdditionalLabels", &additional_lables.join(":"));
        self
    }
}
impl <Q: CanAddReturn> Neo4gMatchStatement<Q> {
    pub fn add_to_return(mut self) -> Self {
        // println!("ret incoming rel_num: {}", self.relation_number);
        if let Some((mut name, entity_type)) = self.previous_entity.clone() {
            // println!("name: {}", &name);
            name = name.replace(":AdditionalLabels", "");
            self.return_refs.push((name, entity_type));
        }
        // println!("ret outgoing rel_num: {}", self.relation_number);
        self
    }
}
impl <Q: PossibleStatementEnd> Neo4gMatchStatement<Q> {
    /// Generates a WHERE call
    /// # Example
    /// ```rust
    /// .filter(Where::new()
    ///     .condition(&node1, prop!(node1.prop1), CompareOperator::Eq)
    ///     .join(CompareJoiner::And)
    ///     .condition(&node1, |_| Node1Props::Prop2(val), CompareOperator::Gt)       
    /// )
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// WHERE node1alias.prop1 = $where1_prop1 AND node1alias.prop2 > $where2_prop2
    /// ```
    /// and asociated params for the inner builder.
    pub fn filter(mut self, filter: Where<Condition>) -> Self {
        if self.where_str.is_empty() {
            self.where_str.push_str("\nWHERE ")
        }
        let (query_part, where_params) = filter.build();
        self.where_str.push_str(&query_part);
        self.params.extend(where_params);
        self
    }
    /// Generates a SET call
    /// # Example
    /// ```rust
    /// .set(&entity1, props!(entity1 => entity1.prop1, Entity1Props::Prop2(val)))
    /// .set(&entity2, props!(entity2 => entity2.prop1, Entity2Props::Prop2(val)))
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// SET entity1alias.prop1 = $set1_prop1, entity1alias.prop2 = $set1_prop2, entity2alias.prop1 = $set2_prop1, entity2alias.prop2 = $set2_prop2
    /// ```
    /// and asociated params for the inner builder.
    pub fn set<T, F>(mut self, entity: &T, props_macro: F) -> Self
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> Vec<T::Props> {
        self.set_number += 1;
        let props = props_macro(entity);
        let alias = entity.get_alias();
        let mut query = String::new();
        let mut params = std::collections::HashMap::new();
        let props_str: Vec<String> = props
            .iter()
            .map(|prop| {
                let (key, value) = prop.to_query_param();
                params.insert(format!("set_{}{}", key.to_string(), self.set_number), value);
                format!("{}.{} = $set_{}{}", alias, key, key, self.set_number)
            })
            .collect();
        query.push_str(&props_str.join(", "));
        self.params.extend(params);
        if self.set_str.is_empty() {
            self.set_str = "\nSET ".to_string();
        } else {
            self.set_str.push_str(", ");
        }
        self.set_str.push_str(&query);
        self
    }
    /// Finalises the current statement, tidies up placeholders, and changes the state of the builder so that new statements can be added.
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
                self.query.push_str(&format!("\nWITH {}", return_aliases.join(", ")));
            }
        }
        self.query = self.query.replace(":AdditionalLabels", "");
        Neo4gBuilder::from(self)
    }
}

//Statement combiners
impl <Q: PossibleQueryEnd> Neo4gBuilder<Q> {
    /// Builds the query and params. This is used by .call(), and should otherwise not be used unless you know what you're doing. 
    /// It has to be a pub fn to allow .call() to work as intended, but is not intended for use by API consumers.
    pub fn build(self) -> (String, HashMap<String, BoltType>) {
        (self.query, self.params)
    }
    /// An alternative to calling .add_to_return() for each object in the query. 
    /// This is a more traditional way of managing returns and may be more familiar to people who are used to writing database queries.
    /// # Example
    /// ```rust
    /// .set_returns(&[(EntityType::Node, node.wrap()), (EntityType::Relation, relation.wrap())])
    /// ```
    /// When .run_query(graph).await; is called, the following will be appended to the query:
    /// ```rust
    /// RETURN node1alias, rel1alias
    /// ```
    pub fn set_returns<T: WrappedNeo4gEntity>(mut self, returns: &[(EntityType, T)]) -> Self {
        if returns.is_empty() && self.return_refs.is_empty() {
            // println!("Nothing will be returned from this query...");
        } else {
            
        }
        if !returns.is_empty() {
            self.return_refs = returns.iter().map(|(entity_type, wrapper)| {
                let entity = wrapper.clone();
                let alias = entity.get_alias();
                (alias, entity_type.clone())
            }).collect();
        }
        self
    }

    /// Generates a SKIP call. 
    /// # Example
    /// ```rust
    /// .skip(5)
    /// ```
    /// The example above generates the following text:
    /// ```rust
    /// SKIP 5
    /// ```
    pub fn skip(mut self, skip: u32) -> Self {
        self.query.push_str(&format!("SKIP {}\n", skip));
        self
    }
    /// Generates a LIMIT call. 
    /// # Example
    /// ```rust
    /// .limit(5)
    /// ```
    /// The example above generates the following text:
    /// ```rust
    /// LIMIT 5
    /// ```
    pub fn limit(mut self, limit: u32) -> Self {
        self.query.push_str(&format!("LIMIT {}\n", limit));
        self
    }
    /// Generates an ORDER BY call
    /// # Example
    /// ```rust
    /// .order_by(&mut entity, &entity.prop, Order::Asc)
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// ORDER BY entityalias.prop1
    /// ```
    /// and asociated params for the inner builder.
    pub fn order_by<T, F>(mut self, entity: &mut T, order: Order, prop_macro: F) -> Self
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> T::Props {
        if self.order_by_str.is_empty() {
            self.order_by_str = "\nORDER BY ".to_string();
        }
        let prop = prop_macro(entity);
        let (name, _) = prop.to_query_param();
        let alias = entity.get_alias();
        self.order_by_str.push_str(&format!("{}.{} {}", alias, &name, order.to_string()));
        self
    }
    /// Runs the query against a provided Graph and returns the registered return objects.
    /// # Example:
    /// ```rust
    /// .run_query(graph, EntityWrapper::from_db_entity).await;
    /// ```
    pub async fn run_query<F, R>(mut self, graph: Graph, unpack: F) -> anyhow::Result<Vec<F::Output>> 
    where F: Fn(DbEntityWrapper) -> R {
        if !self.return_refs.is_empty() {
            self.query.push_str("\nRETURN ");
            let aliases: Vec<&str> = self.return_refs.iter().map(|(alias, _)| alias.as_str()).collect();
            self.query.push_str(&aliases.join(", "));
        }
        self.query.push_str(&self.order_by_str);
        println!("query: {}", self.query.clone());
        println!("params: {:?}", self.params.clone());
        let query = Query::new(self.query).params(self.params);
        let mut return_vec: Vec<R> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            println!("query ran");
            while let Ok(Some(row)) = result.next().await {
                for (alias, entity_type) in &self.return_refs {
                    match entity_type {
                        EntityType::Node => {
                            if let Ok(node) = row.get::<Node>(&alias) {
                                let wrapped_entity = unpack(DbEntityWrapper::Node(node));
                                return_vec.push(wrapped_entity);
                            } else {
                                return Err(anyhow!(format!("Failed to get Node from db for {}", &alias)));
                            }
                        },
                        EntityType::Relation => {
                            if let Ok(relation) = row.get::<Relation>(&alias) {
                                let wrapped_entity = unpack(DbEntityWrapper::Relation(relation));
                                return_vec.push(wrapped_entity);
                            } else {
                                return Err(anyhow!(format!("Failed to get Relation from db for {}", &alias)));
                            }
                        },
                        _ => {
                            return Err(anyhow!(format!("Not a Node or Relation not sure what you were trying to return here, or why...")));
                        }
                    }
                }
            }
        }
        Ok(return_vec)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntityType {
    Node,
    Relation,
    Unwinder,
}

#[derive(Clone, Debug)]
pub enum Clause {
    Create,
    Merge,
    Match,
    Delete,
    None,
}

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
    return_refs: Vec<(String, EntityType)>,
    previous_entity: Option<(String, EntityType)>,
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
    return_refs: Vec<(String, EntityType)>,
    previous_entity: Option<(String, EntityType)>,
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
    return_refs: Vec<(String, EntityType)>,
    previous_entity: Option<(String, EntityType)>,
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
    fn build_inner(self) -> (String, HashMap<String, BoltType>, u32, u32, u32, u32, Vec<(String, EntityType)>) {
        (self.query, self.params, self.node_number, self.relation_number, self.unwind_number, self.set_number, self.return_refs)
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

#[derive(Debug, Clone, Default)]
pub struct Unwinder {
    alias: String,
    array: Array,
}

impl Unwinder {
    pub fn new(array: &Array) -> Self {
        Self {
            alias: String::new(),
            array: array.clone(),
        }
    }
    /// Builds the query and params. This is used by .unwind(), and should otherwise not be used unless you know what you're doing. 
    fn unwind(&mut self) -> (String, HashMap<String, BoltType>) {
        let mut params = HashMap::new();
        let mut query = String::new();
        if !self.array.is_built {
            params.insert(self.array.alias.clone(), self.array.list().into());
            query.push_str(&format!("\nUNWIND ${} as {}", &self.array.alias, &self.alias));
        } else {
            query.push_str(&format!("\nUNWIND {} as {}", &self.array.alias, &self.alias));
        }
        (query, params)
    }
}

impl Aliasable for Unwinder {
    fn get_alias(&self) -> String {
        self.alias.clone()
    }
    fn set_alias(&mut self, alias: &str) {
        self.alias = alias.to_string();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Array {
    alias: String,
    list: Vec<BoltType>,
    is_built: bool,
}

impl Array {
    pub fn new(alias: &str, list: Vec<BoltType>) -> Self {
        Self {
            alias: alias.to_string(),
            list,
            is_built: false,
        }
    }
    fn build(&mut self) -> (String, HashMap<String, BoltType>) {
        if self.is_built {
            return (self.get_alias(), HashMap::new());
        } else {
            self.is_built = true; 
            return (format!("${} AS {}", &self.alias, &self.alias), HashMap::from([(self.alias.clone(), BoltType::from(self.list.clone()))]));
        }
    }
    pub fn list(&self) -> Vec<BoltType> {
        self.list.clone()
    }
}

impl Aliasable for Array {
    fn get_alias(&self) -> String {
        self.alias.clone()
    }
    fn set_alias(&mut self, alias: &str) -> () {
        self.alias = alias.to_string();
    }
}

#[derive(Debug, Clone)]
pub struct With<State> {
    string: String,
    params: HashMap<String, BoltType>,
    with_number: u32,
    _state: PhantomData<State>,
}

impl With<Empty> {
    pub fn new() -> Self {
        Self {
            string: String::new(),
            params: HashMap::new(),
            with_number: 0,
            _state: PhantomData,
        }
    }
}

impl <S> With<S> {
    fn transition<NewState>(self) -> With<NewState> {
        let With {string, params, with_number, ..} = self;
        With {string, params, with_number, _state: std::marker::PhantomData,}
    }
}
impl <CanBuild> With<CanBuild> {
    /// Generates comma separated entity aliases.
    /// # Example
    /// ```rust
    /// .entities(wrap![entity1, entity2])
    /// ```
    /// The example above generates `entity1alias, entity2alias`.
    /// If this was called after other With methods, a comma is also inserted at the start of the string.
    pub fn entities<T: WrappedNeo4gEntity>(mut self, entities: &[T]) -> With<Condition> {
        if entities.len() == 0 {
            return self.transition::<Condition>();
        }
        self.with_number += 1;
        let aliases: Vec<String> = entities.iter().map(|entity| {
            entity.get_alias()
        }).collect();
        if !self.string.is_empty() {
            self.string.push_str(", ");
        }
        self.string.push_str(&aliases.join(", "));
        self.transition::<Condition>()
    }
    /// Generates comma separated array params AS aliases.
    /// # Example
    /// ```rust
    /// .arrays(arrays![array1, array2])
    /// ```
    /// The example above generates `$array1 AS array1, $array2 AS array2`.
    /// If this was called after other With methods, a comma is also inserted at the start of the string.
    pub fn arrays(mut self, arrays: &mut [&mut Array]) -> With<Condition> {
        self.with_number += 1;
        let aliases: Vec<String> = arrays.iter_mut().map(|array|{
            let (string, params) = array.build();
            self.params.extend(params);
            string
        }).collect();
        if !self.string.is_empty() {
            self.string.push_str(", ");
        }
        self.string.push_str(&aliases.join(", "));
        self.transition::<Condition>()
    }
    /// Generates comma separated calls to collect().
    /// # Example
    /// ```rust
    /// .collect(wrap![entity1, entity2])
    /// ```
    /// The example above generates `collect(entit1alias) AS collected_entity1alias1, collect(entity2alias) AS collected_entity2alias2`.
    /// If this was called after other With methods, a comma is also inserted at the start of the string.
    pub fn collect<A: Aliasable>(mut self, entities: &[&A]) -> With<Condition> {
        if !self.string.is_empty() {
            self.string.push_str(", ");
        }
        let strings:Vec<String> = entities.iter().map(|entity| {
            let alias = entity.get_alias();
            self.with_number += 1;
            format!("collect({}) AS collected_{}{}", &alias, &alias, self.with_number)
        }).collect();
        self.string.push_str(&strings.join(", "));
        self.transition::<Condition>()
    }
    fn build(self) -> (String, HashMap<String, BoltType>) {
        (self.string, self.params)
    }
}

impl With<Condition> {
    /// Generates a WHERE call
    /// # Example
    /// ```rust
    /// .filter(Where::new()
    ///     .is_not_null(&entity1)
    ///     .join(CompareJoiner::And)
    ///     .size(&entity2, CompareOperator::Gt(0))       
    /// )
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// WHERE entity1alias IS NOT NULL AND size(entity2alias) > $where_entity2alias1
    /// ```
    /// and asociated params.
    pub fn filter(mut self, filter: Where<Condition>) -> With<ReturnSet> {
        self.string.push_str(" WHERE ");
        let (query_part, where_params) = filter.build();
        self.string.push_str(&query_part);
        self.params.extend(where_params);
        self.transition::<ReturnSet>()
    }
}

#[derive(Debug, Clone)]
pub struct Where<State> {
    string: String,
    params: HashMap<String, BoltType>,
    condition_number: u32,
    _state: PhantomData<State>,
}

impl<S> Where<S> {
    fn transition<NewState>(self) -> Where<NewState> {
        let Where {string, params, condition_number, ..} = self;
        Where {string, params, condition_number, _state: std::marker::PhantomData,}
    }
    fn build_inner(self) -> (String, HashMap<String, BoltType>, u32) {
        (self.string, self.params, self.condition_number)
    }
}

impl Where<Empty> {
    pub fn new() -> Self {
        Self {
            string: String::new(),
            params: HashMap::new(),
            condition_number: 0,
            _state: PhantomData,
        }
    }
    fn new_with_parent<S>(parent: &Where<S>) -> Self {
        Self {
            string: String::new(),
            params: HashMap::new(),
            condition_number: parent.condition_number,
            _state: PhantomData,
        }
    }
}

impl<Q: CanCondition> Where<Q> {
    /// Appends NOT to the string. 
    pub fn not(mut self) -> Self {
        self.string.push_str("NOT ");
        self
    }
    /// Generates a condition string.
    /// # Example
    /// ```rust
    /// .condition(&entity, prop!(entity.prop), CompareOperator::Eq)
    /// ```
    /// The example above generates the following string:
    /// ```rust
    /// entityalias.prop = $where_prop11
    /// ```
    /// and asociated params.
    /// 
    pub fn condition<T, F>(mut self, entity: &T, prop_macro: F, operator: CompareOperator) -> Where<Condition>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> T::Props {
        self.condition_number += 1;
        let prop = prop_macro(entity);
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity.get_alias();
        // println!("condition_alias: {}", &alias);
        self.string.push_str(&format!("{}.{} {} ${}", &alias, name, operator.to_string(), &param_name));
        if let CompareOperator::In(v) = operator {
            self.params.insert(param_name, v.into());
        } else {
            self.params.insert(param_name, value);
        }
        // println!("{}: number{}", alias, self.condition_number);
        self.transition::<Condition>()
    }
    /// Generates a call to the size() cypher function.
    /// # Example
    /// ```rust
    /// .size(&entity, CompareOperator::Gt, 0)
    /// ```
    /// The example above generates `size(entityalias) > $size_entityalias1`
    pub fn size<T: Neo4gEntity>(mut self, entity: &T, operator: CompareOperator, value: i32) -> Where<Condition> {
        self.condition_number += 1;
        let alias = entity.get_alias();
        let param_string = format!("size_{}{}", &alias, self.condition_number);
        self.string.push_str(&format!("size({}) {} ${}", &alias, operator, &param_string));
        self.params.insert(param_string, value.into());
        self.transition::<Condition>()
    }
    /// Generates a condition string for an entity not being null.
    /// # Example
    /// ```rust
    /// .is_not_null(&entity)
    /// ```
    /// The example above generates `entityalias IS NOT NULL`
    pub fn is_not_null<T: Neo4gEntity>(mut self, entity: &T) -> Where<Condition> {
        self.condition_number += 1;
        self.string.push_str(&format!("{} IS NOT NULL", entity.get_alias()));
        self.transition::<Condition>()
    }
    /// Generates a condition string for an entity being null.
    /// # Example
    /// ```rust
    /// .is_null(&entity)
    /// ```
    /// The example above generates `entityalias IS NULL`
    pub fn is_null<T: Neo4gEntity>(mut self, entity: &T) -> Where<Condition> {
        self.condition_number += 1;
        self.string.push_str(&format!("{} IS NULL", entity.get_alias()));
        self.transition::<Condition>()
    }
    /// Generates a condition string with the neo4j coalesce function included.
    /// # Example
    /// ```rust
    /// .coalesce(&entity, prop!(entity.prop), CompareOperator::Eq)
    /// ```
    /// The example above generates the following string:
    /// ```rust
    /// entityalias.prop = coalesce($where_prop1, entityalias.prop)
    /// ```
    /// and asociated params.
    pub fn coalesce<T, F>(mut self, entity: &T, prop_macro: F) -> Where<Condition>
    where T: Neo4gEntity, T::Props: Clone, F: FnOnce(&T) -> T::Props {
        self.condition_number += 1;
        let prop = prop_macro(entity);
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity.get_alias();
        self.string.push_str(&format!("{}.{} = coalesce(${}, {}.{}", alias, name, param_name, alias, name));
        self.params.insert(param_name, value);
        self.transition::<Condition>()
    }
    /// Nests conditions within the inner_builder in parens.
    /// # Example
    /// ```rust
    /// .nest(|inner| {inner
    ///     .condition(&entity1, prop!(entity1.prop1), CompareOperator::Eq)
    ///     .join(CompareJoiner::And)
    ///     .condition(&entity2, |_| Entity2Props::Prop2(val), CompareOperator::Ne)
    /// })
    /// ```
    /// The example above generates "(entity1alias.prop1 = $where_prop11 AND entity2alias.prop2 <> $where_prop22)"
    pub fn nest<F>(mut self, inner_builder_closure: F) -> Where<Condition>
    where F: FnOnce(Where<Empty>) -> Where<Condition> {
        let inner_builder = Where::new_with_parent(&self);
        let (query,
            params,
            condition_number
        ) = inner_builder_closure(inner_builder).build_inner();
        self.condition_number = condition_number;
        self.string.push_str(&format!("({})", query));
        self.params.extend(params);
        self.transition::<Condition>()
    }
}

impl<Q: CanJoin> Where<Q> {
    /// Appends the joiner to the filter string.
    /// # Example
    /// ```rust
    /// .join(CompareJoiner::And)
    /// ```
    pub fn join(mut self, joiner: CompareJoiner) -> Where<Joined> {
        self.string.push_str(&format!(" {} ", joiner.to_string()));
        self.transition::<Joined>()
    }
}

impl Where<Condition> {
    /// Builds the filter and params. This is used by .filter(), and should otherwise not be used unless you know what you're doing. 
    fn build(self) -> (String, HashMap<String, BoltType>) {
        (self.string, self.params)
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
    In(Vec<BoltType>),
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
            CompareOperator::In(v) => "IN", //, v.iter().map(|i| format!("{}", i)).collect::<Vec<String>>().join(", ")),
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

#[derive(Debug, Clone)]
pub enum DbEntityWrapper {
    Node(Node),
    Relation(Relation),
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
pub enum Expr {
    Raw(String),
    Func(Function),
}

#[derive(Debug, Clone)]
pub enum Function {
    Id(Box<Expr>),
    Coalesce(Vec<Expr>),
    Exists(Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Raw(s) => write!(f, "{}", s), // instead of being a raw string, should this be a struct? Alternatively, should the Expr::new() take an Aliasable?
            Expr::Func(func) => write!(f, "{}", func),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Id(expr) => write!(f, "id({})", expr),
            Function::Coalesce(exprs) => {
                let joined = exprs.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "coalesce({})", joined)
            }
            Function::Exists(expr) => write!(f, "exists({})", expr),
        }
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
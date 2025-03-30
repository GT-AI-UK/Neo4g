//use crate::entity_wrapper::{EntityWrapper, PropsWrapper, Label};
use neo4rs::{query, BoltNull, BoltType, Graph, Node, Query, Relation};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
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
    ////return_refs: Vec<(String, EntityType, EntityWrapper)>,
    order_by_str: String,
    ////previous_entity: Option<(String, EntityType, EntityWrapper)>,
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
            ////return_refs: Vec::new(),
            order_by_str: String::new(),
            //previous_entity: None,
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
    ///     .node(&mut node1, &[&Node1Props::Prop(123)]).add_to_return()
    ///     .relation(&mut rel, &[]).add_to_return()
    ///     .node(&mut node2, &[&node2.prop]).add_to_return()
    ///     .on_create()
    ///         .set(node1, &[&Node1Props::Eg(987)]))
    ///         .set(node2, &[&node2.eg]))
    ///     .on_match()
    ///         .set(node1, &[&Node1Props::Eg(321)]))
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// MERGE (node1alias:Node1Label {prop: $node1_prop1)-[relalias:REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// ON CREATE SET node1alias.eg = $node1alias_eg1, node2alias.eg = $node2alias_eg2
    /// ON MATCH SET node1alias.eg = $node1alias_eg1
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
    /// Generates a MATCH statement. 
    /// # Example
    /// ```rust
    /// .get()
    ///     .node(&mut node1, &[&node1.prop1]).add_to_return()
    ///     .relation(&mut rel, &[]).add_to_return()
    ///     .node(&mut node2, &[&Node2Props::Prop(123)]).add_to_return()
    ///     .filter(Where::new()
    ///         .condition(&node1, &Node1Props::Prop2(123), CompareOperator::Gt)         
    ///     )
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// MATCH (node1alias: Node1Label {prop: $node1_prop1})-[relalias: REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// WHERE node1alias.prop2 = $where_prop21
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
    ///     .node(&mut node1, &[&node1.prop1]).add_to_return()
    ///     .relation(&mut rel, &[]).add_to_return()
    ///     .node(&mut node2, &[&Node2Props::Prop(123)]).add_to_return()
    ///     .filter(Where::new()
    ///         .condition(&node1, &Node1Props::Prop2(123), CompareOperator::Gt)         
    ///     )
    /// .end_statement()
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// OPTION MATCH (node1alias: Node1Label {prop: $node1_prop1})-[relalias: REL_TYPE]->(node2alias: Node2Label {prop: $node2_prop2})
    /// WHERE node1alias.prop2 = $where_prop21
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
    /// Generates an UNWIND call. 
    /// # Example
    /// ```rust
    /// .unwind(
    ///     Unwinder::new(vec![PropsWrapper::EntityProps(EntityProps::Prop(123))])
    /// )
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// UNWIND $neo4g_unwind1 as neo4g_unwind1
    /// ```
    /// and asociated params.
    pub fn unwind(mut self, mut unwinder: Unwinder) -> Self {
        self.unwind_number += 1;
        unwinder.alias = format!("neo4g_unwind{}", self.unwind_number);
        let (query, params) = unwinder.unwind();
        self.query.push_str(&format!("\n{}", query));
        self.params.extend(params);
        self
    }
    /// Generates a WITH call. 
    /// # Example
    /// ```rust
    /// .with(&[entity1.wrap(), entity2.wrap()])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// WITH entity1alias, entity2alias
    /// ```
    /// and asociated params.
    pub fn with<T: Aliasable>(mut self, entities_to_alias: &[T]) -> Neo4gBuilder<Withed> {
        let aliases: Vec<String> = entities_to_alias.iter().map(|entity| {
            entity.get_alias()
        }).collect();
        self.query.push_str(&format!("\nWITH {}", aliases.join(", ")));
        self.transition::<Withed>()
    }
}

// impl<Q: CanWhere> Neo4gBuilder<Q> {
//     pub fn filter_with(mut self, filter: Where<Condition>) -> Self { // needs to be specific to with... I'd rather not have lots of filters on it...
//         if self.where_str.is_empty() {
//             self.where_str.push_str("\nWHERE ")
//         }
//         let (query_part, where_params) = filter.build();
//         self.where_str.push_str(&format!("{}\n", &query_part));
//         self.params.extend(where_params);
//         self
//     }
// }

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
        //self.//previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
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
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
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
    // pub fn add_to_return(mut self) -> Self {
    //     if let Some((mut name, entity_type, entity)) = //self.//previous_entity.clone() {
    //         name = name.replace(":AdditionalLabels", "");
    //         //self.//return_refs.push((name, entity_type, entity));
    //     }
    //     self
    // }
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
    /// .node(&mut node, &[&node.prop1, &NodeProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias:NodeLabel {prop1: $node1_prop1, prop2: $node1_prop2})
    /// ```
    /// and asociated params.
    pub fn node<T: Neo4gEntity, P: CurrentProp>(mut self, entity: &mut T, filter_props: &[P]) -> Neo4gMergeStatement<CreatedNode>
    where P: Clone {
        self.node_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.node_number);
        entity.set_alias(&alias);
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
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
    /// .relation(0, &mut relation, &[&relation.prop1, &RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE*0 {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relations<T: Neo4gEntity, P: CurrentProp>(mut self, min_hops: u32, entity: &mut T, filter_props: &[P]) -> Neo4gMergeStatement<CreatedRelation>
    where P: Clone {
        self.relation_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Generates a relation query object. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, &[&relation.prop1, &RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relation<T: Neo4gEntity, P: CurrentProp>(mut self, entity: &mut T, filter_props: &[P]) -> Neo4gMergeStatement<CreatedRelation>
    where P: Clone {
        self.relation_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("*min_hops..", ""));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Generates a relation query object with the arrow going right to left. 
    /// Uses the T::Props vec to set the conditions for the MERGE.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, &[&relation.prop1, RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// <-[realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]-
    /// ```
    /// and asociated params.
    pub fn relation_flipped<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gMergeStatement<CreatedRelation>
    { //where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.relation_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
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
    // pub fn add_to_return(mut self) -> Self {
    //     if let Some((mut name, entity_type, entity)) = //self.//previous_entity.clone() {
    //         name = name.replace(":AdditionalLabels", "");
    //         //self.//return_refs.push((name, entity_type, entity));
    //     }
    //     self
    // }
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
    /// .set(&Entity, &[&Entity1Props::Prop1(123), &Entity1Props::Prop2(456)])
    /// .set(&Entity2, &[&Entity2Props::Prop1(987), &Entity2Props::Prop2(654)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// SET entity1alias.prop1 = $set1_prop1, entity1alias.prop2 = $set1_prop2, entity2alias.prop1 = $set2_prop1, entity2alias.prop2 = $set2_prop2
    /// ```
    /// and asociated params for the inner builder.
    pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: &T, props: &[&T::Props]) -> Self {
        //where T::Props: Clone, PropsWrapper: From<<T as Neo4gEntity>::Props> {
        self.set_number += 1;
        let alias = entity_to_alias.get_alias();
        // let wrapped_props: Vec<PropsWrapper> = props.iter().map(|p| {
        //     PropsWrapper::from(p.to_owned().clone())
        // }).collect();
        // let refed_props: Vec<&PropsWrapper> = wrapped_props.iter().map(|prop| prop).collect();
        // let (query, params) = PropsWrapper::set_by(&alias, self.set_number, &refed_props);
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
    // pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: &T, props: &[&T::Props]) -> Self
    //     where T::Props: Clone, PropsWrapper: From<<T as Neo4gEntity>::Props> {
    //     self.set_number += 1;
    //     let wrapped_props: Vec<PropsWrapper> = props.iter().map(|p| {
    //         PropsWrapper::from(p.to_owned().clone())
    //     }).collect();
    //     let refed_props: Vec<&PropsWrapper> = wrapped_props.iter().map(|prop| prop).collect();
    //     let alias = entity_to_alias.get_alias();
    //     let (query, params) = PropsWrapper::set_by(&alias, self.set_number, &refed_props);
    //     self.params.extend(params);
    //     match self.current_on_str {
    //         OnString::Create => {
    //             if self.on_create_str == "\nON CREATE".to_string() {
    //                 self.on_create_str.push_str("\nSET ");
    //             } else {
    //                 self.on_create_str.push_str(", ");
    //             }
    //             self.on_create_str.push_str(&query)
    //         },
    //         OnString::Match => {
    //             if self.on_match_str == "\nON MATCH".to_string() {
    //                 self.on_match_str.push_str("\nSET ");
    //             } else {
    //                 self.on_match_str.push_str(", ");
    //             }
    //             self.on_match_str.push_str(&query)
    //         },
    //         OnString::None => (),
    //     }
    //     self
    // }
    /// Finalises the current statement, tidies up placeholders, and changes the state of the builder so that new statements can be added.
    pub fn end_statement(mut self) -> Neo4gBuilder<CreatedNode> {
        self.query = self.query.replace(":AdditionalLabels", "");
        //println!("INSIDE MERGE! Query: {}", &self.query);
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
    /// .node(&mut node, &[&node.prop1, &NodeProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// (nodealias:NodeLabel {prop1: $node1_prop1, prop2: $node1_prop2})
    /// ```
    /// and asociated params.
    pub fn node<T: Neo4gEntity, P: CurrentProp>(mut self, entity: &mut T, filter_props: &[P]) -> Neo4gMatchStatement<MatchedNode>
    where P: Clone {
        self.node_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.node_number);
        entity.set_alias(&alias);
        let name = format!("{}{}:AdditionalLabels", label.to_lowercase(), self.node_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Node, EntityWrapper::from(entity.clone())));
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
    /// .relation(0, &mut relation, &[&relation.prop1, &RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE*0 {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relations<T: Neo4gEntity, P: CurrentProp>(mut self, min_hops: u32, entity: &mut T, filter_props: &[P]) -> Neo4gMatchStatement<CreatedRelation>
    where P: Clone {
        self.relation_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("min_hops", &format!("{}", min_hops)));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Generates a relation query object. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, &[&relation.prop1, &RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// [realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]->
    /// ```
    /// and asociated params.
    pub fn relation<T: Neo4gEntity, P: CurrentProp>(mut self, entity: &mut T, filter_props: &[P]) -> Neo4gMatchStatement<MatchedRelation>
    where P: Clone {
        self.relation_number += 1;
        let props: Vec<T::Props> = filter_props.iter().map(|prop| {
            entity.get_current(prop.clone())
        }).collect();
        let label = entity.get_label();
        let alias = format!("{}{}", label.to_lowercase(), self.relation_number);
        entity.set_alias(&alias);
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.entity_by(&alias, &props);
        self.query.push_str(&query_part.replace("*min_hops..", ""));
        self.params.extend(params);
        self.transition::<MatchedRelation>()
    }
    /// Generates a relation query object with the arrow going right to left. 
    /// Uses the T::Props vec to set the conditions for the MATCH.
    /// # Example
    /// ```rust
    /// .relation(&mut relation, &[&relation.prop1, &RelationProps::Prop2(456)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// <-[realtionalias:REL_TYPE {prop1: $relation1_prop1, prop2: $relation1_prop2}]-
    /// ```
    /// and asociated params.
    pub fn relation_flipped<T: Neo4gEntity>(mut self, entity: &mut T) -> Neo4gMatchStatement<CreatedRelation>
    { //where EntityWrapper: From<T>, T: Clone {
        self.relation_number += 1;
        let label = entity.get_label();
        entity.set_alias(&format!("{}{}", label.to_lowercase(), self.relation_number));
        let name = format!("{}{}", label.to_lowercase(), self.node_number);
        //self.//previous_entity = Some((name.clone(), EntityType::Relation, EntityWrapper::from(entity.clone())));
        let (query_part, params) = entity.create_from_self();
        self.query.push_str(&query_part.replace("-[", "<-[").replace("]->", "]-"));
        self.params.extend(params);
        self.transition::<CreatedRelation>()
    }
    /// Provides an empty relation with no direction, simply -- . 
    pub fn relation_undirected(mut self) -> Neo4gMatchStatement<CreatedRelation> {
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

    // pub fn add_to_return(mut self) -> Self {
    //     if let Some((mut name, entity_type, entity)) = //self.//previous_entity.clone() {
    //         name = name.replace(":AdditionalLabels", "");
    //         //self.//return_refs.push((name, entity_type, entity));
    //     }
    //     self
    // }
}
impl <Q: PossibleStatementEnd> Neo4gMatchStatement<Q> {
    /// Generates a WHERE call
    /// # Example
    /// ```rust
    /// .filter(Where::new()
    ///     .condition(&node1, &node1.prop1, CompareOperator::Eq)
    ///     .join(CompareJoiner::And)
    ///     .condition(&node1, &Node1Props::Prop2(456), CompareOperator::Gt)       
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
    /// .set(Entity, &[&Entity1Props::Prop1(123), &Entity1Props::Prop2(456)])
    /// .set(Entity2, &[&Entity2Props::Prop1(987), &Entity2Props::Prop2(654)])
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// SET entity1alias.prop1 = $set1_prop1, entity1alias.prop2 = $set1_prop2, entity2alias.prop1 = $set2_prop1, entity2alias.prop2 = $set2_prop2
    /// ```
    /// and asociated params for the inner builder.
    pub fn set<T: Neo4gEntity>(mut self, entity_to_alias: T, props: &[&T::Props]) -> Self {
        //where T::Props: Clone, PropsWrapper: From<<T as Neo4gEntity>::Props> {
        self.set_number += 1;
        let alias = entity_to_alias.get_alias();
        // let wrapped_props: Vec<PropsWrapper> = props.iter().map(|p| {
        //     PropsWrapper::from(p.to_owned().clone())
        // }).collect();
        // let refed_props: Vec<&PropsWrapper> = wrapped_props.iter().map(|prop| prop).collect();
        // let (query, params) = PropsWrapper::set_by(&alias, self.set_number, &refed_props);
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
            // if !//self.//return_refs.is_empty() {
            //     let return_aliases: Vec<String> = //self.//return_refs.iter().map(|item| {
            //         item.0.clone()
            //     }).collect();
            //     self.query.push_str(&format!("WITH {}\n", return_aliases.join(", ")));
            // }
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
    /// .set_returns(&[(EntityType::Node, EntityWrapper::Node1(node)), (EntityType::Relation, relation.clone().into())])
    /// ```
    /// When .run_query(graph).await; is called, the following will be appended to the query:
    /// ```rust
    /// RETURN node1alias, rel1alias
    /// ```
    // pub fn set_returns(mut self, returns: &[(EntityType, EntityWrapper)]) -> Self {
    //     if returns.is_empty() && //self.//return_refs.is_empty() {
    //         //println!("Nothing will be returned from this query...");
    //     } else {
            
    //     }
    //     if !returns.is_empty() {
    //         //self.//return_refs = returns.iter().map(|(entity_type, wrapper)| {
    //             let entity = wrapper.clone();
    //             let alias = entity.get_alias();
    //             (alias, entity_type.clone(), wrapper.clone())
    //         }).collect();
    //     }
    //     if !//self.//return_refs.is_empty() {
    //         self.query.push_str("RETURN ");
    //         let aliases: Vec<String> = //self.//return_refs.iter().map(|(_, _, wrapper)| {
    //             let entity = wrapper.clone();
    //             let alias = entity.get_alias();
    //             alias
    //         }).collect();
    //         self.query.push_str(&aliases.join(", "));
    //     }
    //     self
    // }

    /// Generates a CALL call
    /// # Example
    /// ```rust
    /// .call(|parent_builder| parent_builder,
    ///     &[&node1, &unwinder1, &relation1],
    ///     Neo4gBuilder::new(
    ///         ... //get(), merge(), create(), etc.
    ///     )
    /// )
    /// ```
    /// The example above generates the following query:
    /// ```rust
    /// CALL (node1alias, unwinder1alias, relation1alias) {
    ///     ... // query generated by inner builder
    /// }
    /// ```
    /// and asociated params for the inner builder.
    pub fn call<A, F, S, B>(mut self, parent_closure: F, entities_to_alias: &[&A], inner_bulder: Neo4gBuilder<B>) -> Neo4gBuilder<Called>
    where A: Aliasable, B: PossibleQueryEnd, F: FnOnce(&Self) -> &Neo4gBuilder<S> {
        let parent = parent_closure(&self);
        let node_number = parent.node_number;
        let relation_number = parent.relation_number;
        let set_number = parent.set_number;
        let unwind_number = parent.unwind_number;
        self.node_number = node_number;
        self.relation_number = relation_number;
        self.set_number = set_number;
        self.unwind_number = unwind_number;
        let aliases: Vec<String> = entities_to_alias.iter().map(|entity| {
            entity.get_alias()
        }).collect();
        let (query, params) = inner_bulder.build();
        self.query.push_str(format!("CALL ({}) {{\n {} \n}}\n", aliases.join(", "), &query).as_str());
        self.params.extend(params);
        self.transition::<Called>()
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
    pub fn order_by<T: Neo4gEntity>(mut self, entity_to_alias: &mut T, prop: &T::Props, order: Order) -> Self {
        if self.order_by_str.is_empty() {
            self.order_by_str = "\nORDER BY ".to_string();
        }
        let (name, _) = prop.to_query_param();
        let alias = entity_to_alias.get_alias();
        self.order_by_str.push_str(&format!("{}.{} {}", alias, &name, order.to_string()));
        self
    }
    /// Runs the query against a provided Graph and returns the registered return objects.
    pub async fn run_query<F, T, R>(mut self, graph: Graph, returns: &[T], unpack: F) -> anyhow::Result<Vec<F::Output>> 
    where T: WrappedNeo4gEntity, F: Fn(DbEntityWrapper) -> R {
        if !returns.is_empty() {
            self.query.push_str("\nRETURN ");
            let aliases: Vec<String> = returns.iter().map(|entity| entity.get_alias()).collect();
            self.query.push_str(&aliases.join(", "));
        }
        self.query.push_str(&self.order_by_str);
        // println!("query: {}", self.query.clone());
        // println!("params: {:?}", self.params.clone());
        let query = Query::new(self.query).params(self.params);
        let mut return_vec: Vec<R> = Vec::new();
        if let Ok(mut result) = graph.execute(query).await {
            //println!("query ran");
            while let Ok(Some(row)) = result.next().await {
                //for (alias, entity_type, ret_obj) in //self.//return_refs.clone() {
                for ret_obj in returns {
                    //println!("attemping to get {} from database. {:?}, {:?}", alias, &entity_type, &ret_obj);
                    match ret_obj.get_entity_type() {
                        EntityType::Node => {
                            if let Ok(node) = row.get::<Node>(&ret_obj.get_alias()) {
                                //println!("got node for: {}", &alias);
                                //let wrapped_entity = EntityWrapper::from_node(node.clone());
                                let wrapped_entity = unpack(DbEntityWrapper::Node(node));
                                return_vec.push(wrapped_entity);
                            } else {
                                //println!("error getting {} from db result", alias);
                            }
                        },
                        EntityType::Relation => {
                            if let Ok(relation) = row.get::<Relation>(&ret_obj.get_alias()) {
                                //println!("got relation for: {}", &alias);
                                let label = relation.typ();
                                //let wrapped_entity = EntityWrapper::from_relation(relation.clone());
                                let wrapped_entity = unpack(DbEntityWrapper::Relation(relation));
                                //println!("wrapped relation: {:?}", wrapped_entity);
                                return_vec.push(wrapped_entity);
                            } else {
                                //println!("error getting {} from db result", alias);
                            }
                        },
                        _ => {}//println!("You've done something strange here...")}
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
    //return_refs: Vec<(String, EntityType, EntityWrapper)>,
    //previous_entity: Option<(String, EntityType, EntityWrapper)>,
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
    //return_refs: Vec<(String, EntityType, EntityWrapper)>,
    //previous_entity: Option<(String, EntityType, EntityWrapper)>,
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
    //return_refs: Vec<(String, EntityType, EntityWrapper)>,
    //previous_entity: Option<(String, EntityType, EntityWrapper)>,
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
            //return_refs,
            order_by_str,
            //previous_entity,
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
            //return_refs,
            order_by_str,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs,
            //previous_entity,
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
            //return_refs: value.//return_refs,
            //previous_entity: value.//previous_entity,
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
            //return_refs: value.//return_refs,
            //previous_entity: value.//previous_entity,
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
            //return_refs: value.//return_refs,
            //previous_entity: value.//previous_entity,
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
            //return_refs: value.//return_refs,
            order_by_str: String::new(),
            //previous_entity: value.//previous_entity,
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
            //return_refs: value.//return_refs,
            order_by_str: String::new(),
            //previous_entity: value.//previous_entity,
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
            //return_refs: value.//return_refs,
            order_by_str: String::new(),
            //previous_entity: value.//previous_entity,
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
}

impl<Q: CanCondition> Where<Q> {
    /// Generates a condition string.
    /// # Example
    /// ```rust
    /// .condition(&entity, &entity.prop1, CompareOperator::Eq)
    /// ```
    /// The example above generates the following string:
    /// ```rust
    /// entityalias.prop1 = $where_prop11
    /// ```
    /// and asociated params.
    pub fn condition<T: Neo4gEntity>(mut self, entity_to_alias: &T, prop: &T::Props, operator: CompareOperator) -> Where<Condition> {
        self.condition_number += 1;
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity_to_alias.get_alias();
        self.string.push_str(&format!("{}.{} {} ${}", alias, name, operator.to_string(), &param_name));
        self.params.insert(param_name, value);
        //println!("{:?}", self.params);
        self.transition::<Condition>()
    }
    /// Generates a condition string with the neo4j coalesce function included.
    /// # Example
    /// ```rust
    /// .coalesce(&entity, &EntityProps::Prop1(123), CompareOperator::Eq)
    /// ```
    /// The example above generates the following string:
    /// ```rust
    /// entityalias.prop1 = coalesce($where_prop11, entityalias.prop1)
    /// ```
    /// and asociated params.
    pub fn coalesce<T: Neo4gEntity>(mut self, entity_to_alias: &T, prop: &T::Props) -> Where<Condition> {
        let (name, value) = prop.to_query_param();
        let param_name = format!("where_{}{}", name, self.condition_number);
        let alias = entity_to_alias.get_alias();
        self.string.push_str(&format!("{}.{} = coalesce(${}, {}.{}", alias, name, param_name, alias, name));
        self.params.insert(param_name, value);
        self.transition::<Condition>()
    }
    // COULD USE Aliasable here for something?
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

    /// Nests conditions within the inner_builder in parens.
    /// # Example
    /// ```rust
    /// .nest(|parent_filter| parent_filter, Where::new_nested()
    ///     .condition(&entity1, EntityProps::Prop(123).into(), CompareOperator::Eq)
    ///     .join(CompareJoiner::And)
    ///     .condition(&entity2, EntityProps::Prop(456).into(), CompareOperator::Ne)
    /// )  
    /// ```
    /// The example above generates "(entity1alias.prop = 123 AND entity2alias.prop <> 456)"
    pub fn nest<F, S>(mut self, parent_closure: F, inner_builder: Where<Condition>) -> Where<Condition>
    where F: FnOnce(&Self) -> &Where<S> {
        let parent = parent_closure(&self);
        self.condition_number = parent.condition_number;
        let (query, params) = inner_builder.build(); // Assuming build() consumes the nested builder into (query, params)
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
    /// It has to be a pub fn to allow .filter() to work as intended, but is not intended for use by API consumers.
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
    list: Vec<BoltType> // need to take a trait maybe? erm... simpler to just take a list of bolttypes?
}

impl Unwinder {
    pub fn new<T: QueryParam>(list: &[T]) -> Self {
        Self {
            alias: String::new(),
            list: list.iter().map(|props_wrapper| {
                let (_, bolt) = props_wrapper.to_query_param();
                bolt
            }).collect(),
        }
    }
    /// Builds the query and params. This is used by .unwind(), and should otherwise not be used unless you know what you're doing. 
    /// It has to be a pub fn to allow .unwind() to work as intended, but is not intended for use by API consumers.
    pub fn unwind(self) -> (String, HashMap<String, BoltType>) {
        let mut params = HashMap::new();
        params.insert(format!("{}", self.alias), self.list.into());
        let query = format!("UNWIND ${} as {}", self.alias, self.alias);
        (query, params)
    }
}

impl Default for Unwinder {
    fn default() -> Self {
        Self {
            alias: String::new(),
            list: Vec::new(),
        }
    }
}

impl Aliasable for Unwinder {
    fn set_alias(&mut self, alias: &str) {
        self.alias = alias.to_string();
    }
    fn get_alias(&self) -> String {
        self.alias.clone()
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
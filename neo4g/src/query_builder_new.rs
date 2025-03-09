use crate::entity_wrapper::EntityWrapper;
use crate::objects::{User, Group};
use crate::traits::Neo4gEntity;
use neo4rs::{BoltNull, BoltType, Graph, Node, Relation, Query};
use std::marker::PhantomData;

Could start Neo4gBuilder again but use statements as functions too?
eg.
Neo4gBuilder::new().create().node(entity).relation(enitty2).node(entity3).ret(e1, e2, e3).run().await?
Neo4gBuilder::new().get().node(entity).relation(en2).node(en3).ret(e1, e3).run().await? //get instead of match
Neo4gBuilder::new().merge(None).node(e1).zero_plus().relation(e2).node(e3).on_match_set(props).on_create_set(props).with(e1, e3).merge(Some(e1, props)).relation(e4).node(e5).run().await?
// unsure whether to have merge take params... can I create a hashmap in the query builder for which nodes are which aliases? Can I validate aliases or are they better as &str?
If doing this, need to have differernt structs to navigate between for each different clause?
Structs/Traits to be in the form: <Clause><PreviousAction>, eg. MergeReferencedNode, MergeReferencedRelation, MatchRefNode, MatchRefRelation

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
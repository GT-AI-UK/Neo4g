use std::collections::HashMap;

use neo4rs::{BoltType, Node, Relation};

use crate::query_builder::{DbEntityWrapper, EntityType};

pub trait WrappedNeo4gEntity: Sized + Aliasable {
    fn from_db_entity(db_entity: DbEntityWrapper) -> Self;
    fn get_entity_type(&self) -> EntityType;
}

pub trait Neo4gLabel: std::fmt::Display {}

pub trait Neo4gEntity: Aliasable {
    type Props: QueryParam;
    fn get_entity_type(&self) -> EntityType;
    fn get_label(&self) -> String;
    fn entity_by(&self, alias: &str, props: &[&Self::Props]) -> (String, std::collections::HashMap<String, BoltType>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
}

pub trait Aliasable {
    fn set_alias(&mut self, alias: &str) -> ();
    fn get_alias(&self) -> String;
}

pub trait QueryParam {
    fn to_query_param(&self) -> (&'static str, BoltType);
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
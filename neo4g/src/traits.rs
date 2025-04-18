use std::collections::HashMap;

use neo4rs::{
    Node,
    Relation,
    BoltType,
    BoltString,
    BoltBoolean,
    BoltInteger,
    BoltFloat,
    BoltList,
    BoltNode,
    BoltRelation,
    BoltUnboundedRelation,
    BoltPoint2D,
    BoltPoint3D,
    BoltBytes,
    BoltPath,
    BoltDuration,
    BoltLocalDateTime,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::query_builder::{DbEntityWrapper, EntityType, Expr, Function};

pub trait WrappedNeo4gEntity: Sized + Aliasable {
    fn from_db_entity(db_entity: DbEntityWrapper) -> Self;
    fn get_entity_type(&self) -> EntityType;
}

pub trait Neo4gLabel: std::fmt::Display {}

pub trait BoltTypeInComparison {
    fn inside(&self) -> String;
}

impl BoltTypeInComparison for BoltString {
    fn inside(&self) -> String {
        format!("{}", &self)
    }
}

// BoltBoolean,
// BoltInteger,
// BoltFloat,
// BoltList,
// BoltNode,
// BoltRelation,
// BoltUnboundedRelation,
// BoltPoint2D,
// BoltPoint3D,
// BoltBytes,
// BoltPath,
// BoltDuration,
// BoltLocalDateTime,

impl BoltTypeInComparison for BoltType {
    fn inside(&self) -> String {
        match &self {
            BoltType::String(v) => v.inside(),
            _ => "not implemented".into(),
        }
    }
}
pub trait Neo4gEntity: Aliasable {
    type Props: QueryParam;
    fn get_entity_type(&self) -> EntityType;
    fn get_label(&self) -> String;
    fn entity_by(&self, alias: &str, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
    fn get_current(&self, prop: &Self::Props) -> Self::Props;
}

pub trait Paramable {
    fn to_query_uuid_param(&self) -> (String, Vec<Uuid>, HashMap<String, BoltType>);
}

pub trait Prop: Default + Clone + std::fmt::Debug + Serialize + for <'a> Deserialize<'a> {}

pub trait Aliasable: std::fmt::Debug + Clone + Paramable {
    fn get_alias(&self) -> String;
    fn set_alias(&mut self, alias: &str) -> ();
    fn get_uuid(&self) -> Uuid;
}

pub trait QueryParam {
    fn to_query_param(&self) -> (&'static str, BoltType);
}

pub trait ToExpr {
    fn to_expr(&self) -> Expr;
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
pub trait CanSetWith {}

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

#[derive(Debug, Clone)]
pub struct WithCondition;

#[derive(Debug, Clone)]
pub struct WithConditioned;

impl CanMatch for Empty {}
impl CanCreate for Empty {}
impl CanDelete for MatchedNode {}
impl CanMatch for WithCondition {}
impl CanCreate for WithCondition {}
impl CanDelete for WithCondition {}
impl CanMatch for WithConditioned {}
impl CanCreate for WithConditioned {}
impl CanDelete for WithConditioned {}
impl CanSetWith for WithCondition {}
impl CanSetWith for Withed {}
//impl CanAddReturn for Withed {}
impl CanWhere for Withed {}
impl CanWith for MatchedNode {}
impl CanWith for CreatedNode {}
impl CanWith for ReturnSet {}
impl CanWith for Called {}
impl CanMatch for Called {}
impl CanCreate for Called {}
impl PossibleQueryEnd for Called {}
impl CanWith for Empty {}
impl PossibleStatementEnd for MatchedNode {}
impl PossibleStatementEnd for CreatedNode {}
impl PossibleStatementEnd for ReturnSet {}
impl PossibleStatementEnd for Condition {}
impl PossibleStatementEnd for DeletedEntity {}
impl PossibleQueryEnd for DeletedEntity {}
impl PossibleQueryEnd for MatchedNode {}
impl PossibleQueryEnd for CreatedNode {}
impl PossibleQueryEnd for Withed {}
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
use neo4g::prelude::*;
use crate::entity_wrapper::{EntityWrapper, Nothing};

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct GroupTemplate {
    id: String,
    name: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct MemberOfTemplate {
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gRelation, Serialize, Deserialize, Debug, Clone)]
pub struct HasComponentTemplate {
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)] //
pub struct UserTemplate {
    id: String,
    name: String,
    #[serde(skip)]
    password: String,
    forename: String,
    surname: String,
    #[not_query_param]
    groups: Vec<GroupTemplate>,
    #[serde(skip)]
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gProp, Serialize, Deserialize, Debug, Clone, Default)]
pub enum ComponentType {
    #[default]
    Type1,
    Type2,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct ComponentTemplate {
    id: String,
    path: String,
    component_type: ComponentType,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

#[derive(Neo4gNode, Serialize, Deserialize, Debug, Clone)]
pub struct PageTemplate {
    id: String,
    path: String,
    #[not_query_param]
    components: Vec<Component>,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    deleted: bool,
}

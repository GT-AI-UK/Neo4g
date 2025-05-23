use crate::entity_wrapper::{EntityWrapper, Label};
use crate::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use chrono::{NaiveDateTime, Utc};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, CompOper, Neo4gBuilder, Where};
use neo4g::traits::WrappedNeo4gEntity;
use neo4g_macro_rules::{no_props, prop, props};
use neo4rs::Graph;
use dotenv::dotenv;
use std::{env, result};
use heck::ToShoutySnakeCase;
use neo4rs::BoltType;

pub async fn connect_neo4j() -> Graph { //return db object, run on startup, bind to state var
    let mut host = String::new();
    let mut port = String::new();
    let mut db_user = String::new();
    let mut db_password = String::new();
    dotenv().ok();
    if let Ok(env_host) = env::var("DB_HOST") {host = env_host.to_string();} else {println!("DB_HOST is not set in the .env file");}
    if let Ok(env_port) = env::var("DB_PORT") {port = env_port.to_string();} else {println!("DB_PORT is not set in the .env file");}
    if let Ok(env_db_user) = env::var("DB_USERNAME") {db_user = env_db_user.to_string();} else {println!("DB_USERNAME is not set in the .env file");}
    if let Ok(env_db_password) = env::var("DB_PASSWORD") {db_password = env_db_password.to_string();} else {println!("DB_PASSWORD is not set in the .env file");}
    let uri = format!("bolt://{}:{}", host, port);
    let graph = Graph::new(&uri, &db_user, &db_password).await.unwrap();
    graph
}

pub async fn query_builder_query_bench() {
    let graph = connect_neo4j().await;
    let mut component1 = Component::new("cid3", "path3", ComponentType::Type1, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut component2 = Component::new("cid73", "path16", ComponentType::Type2, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut hcrel1 = HasComponent::default();
    let mut hcrel2 = HasComponent::default();
    let mut page1 = Page::new("pid4", "ppath4", vec![component1.clone().into(), component2.clone().into()], Utc::now().naive_local(), Utc::now().naive_local(), false);
    let result = Neo4gBuilder::new()
        .get()
            .node(&mut page1, props!(page1 => page1.id))
            .relation(&mut hcrel1, no_props!())
            .node(&mut component1, props!(component1 => component1.id))
            .filter(Where::new()
                .condition_prop(&page1, Some(&page1.id), CompareOperator::by_prop(CompOper::Eq, &page1.id, query_builder::RefType::Val))
            )
        .end_statement()
        .run_query(graph, EntityWrapper::from_db_entity).await;
}
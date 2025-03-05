use crate::Neo4gBuilder;
use neo4rs::{Graph, Query, Node, Relation};
use neo4g_macro_rules::{generate_props_wrapper, generate_entity_wrapper};
use neo4g_derive::Neo4gNode;
use neo4g_derive::Neo4gEntityWrapper;
use neo4g_traits::*;

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

fn main() {
    let graph = connect_neo4j();
    let (query, params) = User::get_node_by(&[UserProps::Name("Test".to_string())]);
    let (query, params) = Group::get_node_by(&[GroupProps::Name("TestG".to_string())]);
    println!("{}", query);
    let user = User::new(12, "Test".to_string());
    println!("{}", user.get_entity_type());
    println!("{:?}", user.clone());
    let test = Neo4gBuilder::new()
        .match_node(&user, &[UserProps::Name("SDF".to_string())])
        .merge_node(&user, &[UserProps::Name("Sasd".to_string())])
        .build();
    println!("{}", test.0);
    let test_user_props = UserProps::Id(15);
    println!("{:?}", test_user_props);
    println!("{}", user.id());
    let binding = EntityWrapper::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
    let test = binding.as_tuple();
    println!("{:?}", test);
    let binding2 = EntityWrapper::Thing(Thing::new(32, "Testthing".to_string()));
    let test2 = binding2.as_tuple();
    println!("{:?}", test2);

    let test3 = Test::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
    println!("{:?}", test3);

    //let another_test = test2.inner_test();
}
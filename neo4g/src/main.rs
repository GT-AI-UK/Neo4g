use neo4g_derive::Neo4gNode;
//use neo4g_derive::Neo4gEntityWrapper;
use neo4g::traits::{Neo4gEntity, Neo4gProp};
use neo4g::objects::{User, Group, UserProps, GroupProps};
use neo4g::entity_wrapper::EntityWrapper;
use neo4g::query_builder::Neo4gBuilder;
use neo4rs::Graph;
use dotenv::dotenv;
use std::env;
use paste::paste;

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
    println!("connected to graph");
    graph
}

#[tokio::main]
async fn main() {
    let graph = connect_neo4j().await;
    let (query, params) = User::get_node_by(&[UserProps::Name("Test".to_string())]);
    let (query, params) = Group::get_node_by(&[GroupProps::Name("TestG".to_string())]);
    println!("{}", query);
    let user = User::new(0, "Test".to_string());
    println!("{}", user.get_entity_type());
    println!("{:?}", user.clone());
    let test1 = Neo4gBuilder::new()
        .match_node(user.clone(), &[UserProps::Name("admin".to_string())])
        //.merge_node(&user, &[UserProps::Name("Sasd".to_string())])
    
        .add_to_return();
        println!("match?: {:?}", test1.clone());
        let test = test1.run_query(graph).await;
    println!("{:?}", test);
    let test_user_props = UserProps::Id(15);
    println!("{:?}", test_user_props);
    let test = EntityWrapper::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
    println!("{:?}", test);

    let test2 = EntityWrapper::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
    // let maybe_user = User::test_unwrap(EntityWrapper::User(user.clone()));
    // println!("{:?}", maybe_user);
    // let maybe_user2 = User::test_unwrap(test2.clone());
    // println!("{:?}", maybe_user2);
    let maybe_user3 = EntityWrapper::User(user);
    println!("{}", maybe_user3 == test2);

}
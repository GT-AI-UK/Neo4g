use neo4rs::{Graph, Query, Node, Relation, BoltType};
use std::collections::HashMap;
use neo4g::Neo4gBuilder;
use objects::{User, Group, Thing};

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
    let graph = connect_neo4j().await;
    let user = User::new(32, "test");
    let return_obj = //something
    let query_test = Neo4gBuilder::new()
        .create_node(user)
        .run_query(graph)
}

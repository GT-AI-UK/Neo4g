use neo4g_derive::Neo4gNode;
//use neo4g_derive::Neo4gEntityWrapper;
use neo4g::traits::{Neo4gEntity, Neo4gProp};
use neo4g::objects::{User, Group, UserProps, GroupProps};
use neo4g::entity_wrapper::EntityWrapper;
use neo4g::query_builder::Neo4gBuilder;
use neo4rs::{query, Graph, Node};
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
    let user = User::new(55, "Test32".to_string(), vec![(Group::new(32, "Nothing happens here".to_string(), "Nothing happens here".to_string()))]);
    let test1 = Neo4gBuilder::new()
        .get()
            .node(user.clone(), &[UserProps::Id(14),UserProps::Name("Test12".to_string())])
            .set("user1", &[UserProps::Id(15).into()])
            .add_to_return()
        .end_statement();
        // .merge()
        //     .node(user.clone(), &[UserProps::Id(55),UserProps::Name("Test32".to_string())])
        //     .on_match().set("user1", &[UserProps::Id(14).into()])
        //     .on_create().set("user1", &[UserProps::Name("Test12".to_string()).into()])
        //     .add_to_return()
        //     .end_statement();
        println!("match?: {:?}", test1.clone());
    let test = test1.run_query(graph).await;
    println!("{:?}", test);
}



// #[tokio::main]
// async fn main() {
//     // let graph = connect_neo4j().await;
//     // let (query1, params) = User::node_by(&[UserProps::Name("Test".to_string())]);
//     // let (query1, params) = Group::node_by(&[GroupProps::Name("TestG".to_string())]);
//     // println!("{}", query1);
//     // let user = User::new(0, "Test3".to_string(), vec![(Group::new(32, "Nothing happens here".to_string(), "Nothing happens here".to_string()))]);
//     // println!("{}", user.get_entity_type());
//     // println!("{:?}", user.clone());
//     let test1 = Neo4gBuilder::new()
//         .r#match()
//             .node(user.clone(), &[UserProps::Id(45),UserProps::Name("Test2345d".to_string())])
//             .set("User1", &[UserProps::Id(12).into()])
//             .add_to_return()
//         .end_statement();
//         //.set_returns(&[]);
//         println!("match?: {:?}", test1.clone());
//         let test = test1.run_query(graph).await;
//     // println!("{:?}", test);
//     // let test_user_props = UserProps::Id(15);
//     // println!("{:?}", test_user_props);
//     // let test = EntityWrapper::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
//     // println!("{:?}", test);

//     // let test2 = EntityWrapper::Group(Group::new(32, "TestG2".to_string(), "asdf".to_string()));
//     // // let maybe_user = User::test_unwrap(EntityWrapper::User(user.clone()));
//     // // println!("{:?}", maybe_user);
//     // // let maybe_user2 = User::test_unwrap(test2.clone());
//     // // println!("{:?}", maybe_user2);
//     // let maybe_user3 = EntityWrapper::User(user);
//     // println!("{}", maybe_user3 == test2);
//     // let mut result = graph.execute(query("MATCH (n) RETURN n")).await.unwrap();
//     // while let Ok(Some(row)) = result.next().await {
//     //     let node = row.get::<Node>("n").unwrap();
//     //     println!("{:?}", node);
//     // }
// }
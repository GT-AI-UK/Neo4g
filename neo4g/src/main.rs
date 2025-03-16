use neo4g::entity_wrapper::EntityWrapper;
use neo4g::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, Neo4gBuilder, Where};
use neo4rs::Graph;
use dotenv::dotenv;
use std::env;
use heck::ToShoutySnakeCase;

pub async fn connect_neo4j() -> Graph { //return db object, run on startup, bind to state var
    let test = "CamalCase".to_shouty_snake_case();
    println!("{}",test);
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

async fn authenticate_user(graph: Graph, identifier: UserProps, password: UserProps) -> bool {
    let mut permissions:Vec<String> = Vec::new();
    let mut pw_string = String::new();
    match identifier {
        UserProps::Id(_) => {},
        UserProps::Name(_) => {},
        _ => {
            println!("unacceptable identifier provided, failed.");
            return false;
        }
    }
    if let UserProps::Password(pw) = password {
        pw_string = pw;
    } else {
        println!("unacceptable password prop provided, failed.");
        return false;
    }
    let result = Neo4gBuilder::new()
        .get().node(User::default(), &[identifier]).add_to_return()
            .relations(0, MemberOf::default(), &[])
            .node(Group::default(), &[])
            .relation(MemberOf::default(), &[]).add_to_return()
            .node(Group::default(), &[]).add_to_return()
            .filter(Where::new()
                .condition("user1", UserProps::Deleted(false).into(), CompareOperator::Eq)
                .join(CompareJoiner::And)
                .condition("member_of2", MemberOfProps::Deleted(false).into(), CompareOperator::Eq)
                .join(CompareJoiner::And)
                .condition("group3", GroupProps::Deleted(false).into(), CompareOperator::Eq)
            )
            .end_statement()
        .run_query(graph).await;
    if let Ok(entities) = result {
        println!("{:?}", entities.clone());
        let (mut users, mut member_ofs, mut groups) = (Vec::new(), Vec::new(), Vec::new());
        for entity in entities {
            match entity {
                EntityWrapper::User(user) => users = vec![user],
                EntityWrapper::MemberOf(member_of) => member_ofs.push(member_of),
                EntityWrapper::Group(group) => groups.push(group),
                _ => {}
            }
        }
        let mut user: User;
        if users.len() == 1 {
            user = users[0].clone();
        } else {
            return false;
        }
        user.groups = groups;
        println!("user: {:?}", user);
        println!("rels: {:?}", member_ofs);
    }
    true
}

#[tokio::main]
async fn main() {
    let graph = connect_neo4j().await;
    let user = User::new(55,
        "Test32".to_string(),
        "password".to_string(),
        "forname".to_string(),
        "surname".to_string(),
        false,
        vec![(Group::new(32, "Nothing happens here".to_string(), false))],
        "asdf".to_string(),
    );
    let test = authenticate_user(graph, UserProps::Name("admin".to_string()), UserProps::Password("asdf".to_string())).await;



    // let test1 = Neo4gBuilder::new()
    //     .get()
    //         .node(user.clone(), &[])
    //         .filter(
    //             Where::new()
    //             .nest(
    //                 Where::new()
    //                 .condition("user1", UserProps::Id(17).into(), CompareOperator::Gt)
    //                 .join(CompareJoiner::And)
    //                 .condition("user1", UserProps::Id(33).into(), CompareOperator::Lt)
    //             )
    //             .join(CompareJoiner::Or)
    //             .condition("user1", UserProps::Id(35).into(), CompareOperator::Eq)
    //         )
    //         .add_to_return()
    //     .end_statement()
    //     .run_query(graph).await;
        
        //println!("match?: {:?}", test1.clone());
   //let test = test1.run_query(graph).await;
    //println!("{:?}", test1);
}

// .merge()
        //     .node(user.clone(), &[UserProps::Id(55),UserProps::Name("Test32".to_string())])
        //     .on_match().set("user1", &[UserProps::Id(14).into()])
        //     .on_create().set("user1", &[UserProps::Name("Test12".to_string()).into()])
        //     .add_to_return()
        //     .end_statement();

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
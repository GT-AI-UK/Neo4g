use neo4g::entity_wrapper::{EntityWrapper, Label};
use neo4g::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, Neo4gBuilder, Where};
use neo4rs::Graph;
use dotenv::dotenv;
use std::{env, result};
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

#[tokio::main]
async fn main() {
    let graph = connect_neo4j().await;
    let mut component1 = Component::new("cid3", "path3", ComponentType::Type1);
    let mut component2 = Component::new("cid73", "path16", ComponentType::Type2);
    let mut hcrel1 = HasComponent::default();
    let mut hcrel2 = HasComponent::default();
    let mut page1 = Page::new("pid4", "ppath4", vec![component1.clone(), component2.clone()]);
    let result = Neo4gBuilder::new()
        .get()
            .node(&mut page1, &[PageProps::Id("pid99".to_string())]).add_to_return()
            .relation(&mut hcrel1, &[]).add_to_return()
            .node(&mut component1, &[ComponentProps::Id("cid3".to_string())]).add_to_return()
        .end_statement()
        .get()
            .node_ref(&page1)
            .relation(&mut hcrel2, &[]).add_to_return()
            .node(&mut component2, &[ComponentProps::Id("cid73".to_string())]).add_to_return()
            .filter(Where::new()
                .condition(&page1, &PageProps::Id("pid99".into()).into(), CompareOperator::Eq)
                .join(CompareJoiner::And)
                .nest(|parent_filter| parent_filter, Where::new()
                    .condition(&component1, &ComponentProps::Id("pid99".into()).into(), CompareOperator::Ne)
                    .join(CompareJoiner::And)
                    .condition(&component2, &ComponentProps::Id("pid99".into()).into(), CompareOperator::Ne)
                )          
            )
            .end_statement()
        .run_query(graph).await;
    println!("{:?}", result);

    let test_filter = Where::new()
        .condition(&page1, &page1.id, CompareOperator::Eq)
        .join(CompareJoiner::And)
        .nest(|p| p, Where::new().condition(&page1, &PageProps::Id("pid99".into()).into(), CompareOperator::Eq));

    // !! Functional MERGE Query:
    // let result = Neo4gBuilder::new()
    // .merge()
    //     .node(&mut page1, &[PageProps::Id("pid99".to_string())]).add_to_return()
    //     .relation(&mut hcrel1, &[]).add_to_return()
    //     .node(&mut component1, &[ComponentProps::Id("cid3".to_string())]).add_to_return()
    //     .on_create()
    //         .set(page1.clone(), &[PageProps::Path("on_create_set page1".to_string())])
    //         .set(component1.clone(), &[ComponentProps::Path("on_create_set c1p1".to_string())])
    //     .on_match()
    //         .set(page1.clone(), &[PageProps::Path("on_match_set page1".to_string())])
    // .end_statement()
    // .with(&[&page1.clone().into(), &component1.clone().into(), &hcrel1.clone().into()])
    // .merge()
    //     .node_ref(&page1)
    //     .relation(&mut hcrel2, &[]).add_to_return()
    //     .node(&mut component2, &[ComponentProps::Id("cid73".to_string())]).add_to_return()
    // .end_statement()
    // .run_query(graph).await;
    // println!("{:?}", result);
    
    // !! Functional CREATE Query:
    // let result = Neo4gBuilder::new()
    //     .create()
    //         .node(&mut page1).add_to_return()
    //         .relation(&mut hcrel1).add_to_return()
    //         .node(&mut component1).add_to_return()
    //         .end_statement()
    //     .with(&[&page1.clone().into(), &component1.clone().into(), &hcrel1.clone().into()])
    //     .create()
    //         .node_ref(&page1)
    //         .relation(&mut hcrel2).add_to_return()
    //         .node(&mut component2).add_to_return()
    //         .end_statement()
    //     .run_query(graph).await;
    // println!("{:?}", result);



        
    //let test1 = Neo4gBuilder::new()
        
//         println!("match?: {:?}", test1.clone());
//    let test = test1.run_query(graph).await;
//     println!("{:?}", test1);
}

// async fn authenticate_user() -> impl IntoResponse { //graph: Graph, identifier: UserProps, password: UserProps
//     let graph = connect_neo4j().await;
//     let (identifier, password) = (UserProps::Name("admin".to_string()), UserProps::Password("asdf".to_string()));
//     let mut permissions:Vec<String> = Vec::new();
//     let mut pw_string = String::new();
//     match identifier {
//         UserProps::Id(_) => {},
//         UserProps::Name(_) => {},
//         _ => {
//             println!("unacceptable identifier provided, failed.");
//             return Json(UserTemplate::from(User::default()));
//         }
//     }
//     if let UserProps::Password(pw) = password {
//         pw_string = pw;
//     } else {
//         println!("unacceptable password prop provided, failed.");
//         return Json(UserTemplate::from(User::default()));
//     }
//     let mut user = User::default();
//     let mut list_intermediary_members = MemberOf::default(); //lists of rels can't be used without unwinding
//     let mut intermediary_groups = Group::default();
//     let mut member_ofs = MemberOf::default();
//     let mut groups = Group::default();
//     let result = Neo4gBuilder::new()
//         .get().node(&mut user, &[identifier]).add_to_return() // Instead of taking entity, take &mut entity? In this way, alias could be stored in the struct?
//             // forward definitions would be required, which may be problematic...
//             // alternatively, could use an internal field of query builder to track each struct provided to each method, but referencing them is complicated?
//             .relations(0, &mut list_intermediary_members, &[])
//             .node(&mut intermediary_groups, &[])
//             .relation(&mut member_ofs, &[]).add_to_return()
//             .node(&mut groups, &[]).add_to_return()
//             .filter(Where::new()
//                 .condition(&user, UserProps::Deleted(false).into(), CompareOperator::Eq)
//                 .join(CompareJoiner::And)
//                 .condition(&member_ofs, MemberOfProps::Deleted(false).into(), CompareOperator::Eq)
//                 .join(CompareJoiner::And)
//                 .condition(&groups, GroupProps::Deleted(false).into(), CompareOperator::Eq)
//             )
//             .end_statement()
//         .run_query(graph).await;
//     if let Ok(entities) = result {
//         println!("{:?}", entities.clone());
//         let (mut users, mut member_ofs, mut groups) = (Vec::new(), Vec::new(), Vec::new());
//         for entity in entities {
//             match entity {
//                 EntityWrapper::User(user) => users = vec![user],
//                 EntityWrapper::MemberOf(member_of) => member_ofs.push(member_of),
//                 EntityWrapper::Group(group) => groups.push(group),
//                 _ => {}
//             }
//         }
//         let mut user: User;
//         if users.len() == 1 {
//             user = users[0].clone();
//         } else {
//             return Json(UserTemplate::from(User::default()));
//         }
//         user.groups = groups;
//         println!("user: {:?}", user);
//         println!("rels: {:?}", member_ofs);
//         return Json(UserTemplate::from(user));
//     }
//     Json(UserTemplate::from(User::default()))
// }

// use axum::{
//     error_handling::HandleErrorLayer,
//     extract::{Path, Query, State},
//     http::StatusCode,
//     response::IntoResponse,
//     routing::{get, patch},
//     Json, Router,
// };

// #[tokio::main]
// async fn main() {
//     // Build the application with a route.
//     let test = Label::Any;
//     println!("{}", test);
//     let app = Router::new()
//         .route("/hello", get(authenticate_user));

//         let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
//         .await
//         .unwrap();
//     axum::serve(listener, app).await.unwrap();
// }


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
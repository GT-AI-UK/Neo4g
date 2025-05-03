use example_consumer::entity_wrapper::{EntityWrapper, Label, ValueProps};
use example_consumer::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};

use dotenv::dotenv; 
use std::{env, vec};
use neo4g::prelude::ssr::*;

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
    let mut component1 = Component::new("cid3", "path3sadf", ComponentType::Type1, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut component2 = Component::new("cid4", "path4", ComponentType::Type2, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut hcrel1 = HasComponent::default();
    let mut hcrel2 = HasComponent::default();
    let mut page1 = Page::new("pid4", "p1sadfpath234", vec![component1.clone().into(), component2.clone().into()], Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut page2 = Page::new("pid99", "DID IT WORK?!", Vec::new(), Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut page3 = Page::new("pid6", "DID IT WORK?!", Vec::new(), Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut array1 = Array::new("array1", vec!["cid3".into(), "cid4".into()]);
    let mut collect_page2 = FunctionCall::from(Function::Collect(Box::new(Expr::from(&page2))));
    let mut groups = Array::new("member_ofs", vec!["582bb0b6-5e9e-4a5e-90ba-9d9a97410166".into(), "e4957a65-4dd2-4c74-b356-a271d6c0982b".into()]);
    let mut user = User::new("8f8c54b6-5d22-45d6-9a24-dfacaa8d37f5", "admin", "8f327a097ce4b035bd0425c9782f756c4b3e6a080bae8ad2b139cbc6c31e6575", "system3", "user3", Vec::new(), Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut member_of = MemberOf::new(Utc::now().naive_local(), Utc::now().naive_local(), false);
    //let mut collect_page2 = FunctionCall::from(Function::Coalesce(vec![Expr::from(Function::Id(Box::new(Expr::from(&page2)))), Expr::from(&page3)]));
    let mut unwound_groups_option_match = Group::default();
    let mut size_groups_fn = Function::Size(Box::new(Expr::from(&groups)));
    let mut unwound_groups = Unwinder::new(&groups);
    let mut collect_groups = FunctionCall::from(Function::Collect(Box::new(Expr::from(&unwound_groups_option_match))));

    //complex, real-world query test
    let result = Neo4gBuilder::new()
        .merge()
            .node(&mut user, props!(user => user.name))
            .on_create()
                .set(&user, props!(user => user.name, user.forename, user.surname, user.password, user.deleted))
            .on_match()
                .set(&user, props!(user => user.forename, user.surname, user.password))
        .end_statement()
        .with()
            .entities(&[user.wrap()])
            .arrays(arrays![groups])
        .call_with(&[user.wrap(), EntityWrapper::Array(groups.clone())], |inner| {
            inner
                .with()
                    .entities(wrap![user])
                    .arrays(arrays![groups])
                    .filter(Where::new()
                        .condition(&size_groups_fn, CompareOperator::by_prop(CompOper::Gt, &ValueProps::Int(0), RefType::Val))
                    )
                .unwind(&mut unwound_groups)
                .optional_match()
                    .nodes_by_unwound(&mut unwound_groups_option_match, prop!(unwound_groups_option_match.id), &unwound_groups)
                .end_statement()
                .with()
                    .entities(wrap![user, unwound_groups_option_match])
                    .filter(Where::new().is_not_null(&unwound_groups_option_match))
                .merge()
                    .node_ref(&user)
                    .relation(&mut member_of, no_props!())
                    .node_ref(&unwound_groups_option_match)
                    .on_create()
                        .set(&member_of, props!(member_of => member_of.created, member_of.updated, member_of.deleted))
                .end_statement()
                .with()
                    .entities(wrap![user])
                    .function(&mut collect_groups)
                .optional_match()
                    .node_ref(&user)
                    .relation(&mut member_of, no_props!())
                    .node_ref(&unwound_groups_option_match)
                    .filter(Where::new()
                        .not()
                        .condition(&unwound_groups_option_match, CompareOperator::by_aliasable(CompOper::In, &collect_groups))
                    )
                    .delete(wrap![member_of], false)
                .end_statement()
        })
        .get()
            .node(&mut user, props!(user => user.name)).add_to_return()
        .end_statement()
        .run_query(graph, EntityWrapper::from_db_entity).await;
                
    println!("{:?}", result);
    // !!Functional MERGE Query:
    // let result = Neo4gBuilder::new()
    // .merge()
    //     .node(&mut page1, props!(page1 => page1.id)).add_to_return()
    //     .relation(&mut hcrel1, no_props!()).add_to_return()
    //     .node(&mut component1, |component1| vec![component1.id.clone()]).add_to_return()
    //     .on_create()
    //         .set(&page1, props!(page1 => PageProps::Path("on_match_set page1".to_string())))
    //         .set(&component1, props!(component1 => ComponentProps::Path("on_match_set component1".to_string())))
    //     .on_match()
    //         .set(&page1, props!(page1 => PageProps::Path("on_match_set page1".to_string())))
    // .end_statement()
    // .with(wrap![page1, component1, hcrel1])
    // .merge()
    //     .node_ref(&page1)
    //     .relation(&mut hcrel2, no_props!())
    //     .node(&mut component2, props!(component2 => component2.id))
    // .end_statement()
    // .run_query(graph, EntityWrapper::from_db_entity).await;
    // println!("{:?}", result);

    // !! Functional MATCH Query:
    // let result = Neo4gBuilder::new()
    //     .get()
    //         .node(&mut page3, props!(page3 => page3.id)).add_to_return()
    //         .set(&page3, props!(page3 => page3.path))
    //     .end_statement()
    //     .call(wrap![page3], |inner| {
    //         inner.get()
    //             .node(&mut page2, props!(page2 => page2.id))
    //             .set(&page2, props!(page2 => page2.path))
    //             .set(&page3, props!(page3 => PageProps::Path("TEST!!!".into())))
    //         .end_statement()
    //     })
    //     .with()
    //         .entities(&[page3.wrap()])
    //         .arrays(arrays![array1])
    //         .function(&mut collect_page2)
    //     .unwind(&mut Unwinder::new(&array1))
    //     .get()
    //         .node(&mut page1, props!(page1 => page1.id)).add_to_return()
    //         .relation(&mut hcrel1, no_props!()).add_to_return()
    //         .node(&mut component1, |component1| vec![component1.id.clone()]).add_to_return()
    //         .set(&hcrel1, props!(hcrel1 => hcrel1.deleted))
    //         .set(&component1, props!(component1 => component1.path))
    //     .end_statement()
    //     .get()
    //         .node_ref(&page1)
    //         .relation(&mut hcrel2, no_props!()).add_to_return()
    //         .node(&mut component2, props!(component2 => component2.path, ComponentProps::Id("cid4".to_string()))).add_to_return()
    //         .filter(Where::new()
    //             .nest(|inner| {inner
    //                 .condition(&component1, Some(&component1.id), CompareOperator::Eq)
    //                 .join(CompareJoiner::And)
    //                 .condition(&component2, Some(&ComponentProps::Id("pid99".into())), CompareOperator::Ne)
    //             })
    //             .join(CompareJoiner::And)
    //             .condition(&page1, Some(&PageProps::Id("pid4".into())), CompareOperator::Eq)
    //             .join(CompareJoiner::And)
    //             // .condition_fn_prop(&component1, prop!(component1.id), CompareOperator::Eq, Function::Id(
    //             //     Box::new(
    //             //         Expr::from(Function::Coalesce(vec![Expr::from(&component1), Expr::from(&page1)]))
    //             //     )
    //             // ))
    //             .condition(&component1, Some(&component1.id), CompareOperator::InVec(array1.list()))
    //         )
    //         .end_statement()
    //     .run_query(graph, EntityWrapper::from_db_entity).await;
    // println!("{:?}", result);
    
    
    // !! Functional CREATE Query:
    // let result = Neo4gBuilder::new()
    //     .create()
    //         .node(&mut page1).add_to_return()
    //         .relation(&mut hcrel1).add_to_return()
    //         .node(&mut component1).add_to_return()
    //         .end_statement()
    //     .with(wrap![page1, hcrel1, component1])
    //     .create()
    //         .node_ref(&page1)
    //         .relation(&mut hcrel2).add_to_return()
    //         .node(&mut component2).add_to_return()
    //         .end_statement()
    //     .run_query(graph, EntityWrapper::from_db_entity).await;
    // println!("{:?}", result);

}
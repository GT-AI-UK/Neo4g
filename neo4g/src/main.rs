use neo4g_derive::Neo4gNode;
//use neo4g_derive::Neo4gEntityWrapper;
use neo4g::traits::{Neo4gEntity, Neo4gProp};
use neo4g::objects::{User, Group, UserProps, GroupProps};
use neo4g::entity_wrapper::EntityWrapper;
use neo4g::query_builder::Neo4gBuilder;

use paste::paste;

#[tokio::main]
async fn main() {
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
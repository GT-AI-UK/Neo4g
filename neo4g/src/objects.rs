use std::collections::HashMap;
use neo4rs::{Node, Relation};
use crate::entity_wrapper::EntityWrapper;
use neo4g_derive::Neo4gNode;
use crate::traits::Neo4gEntity;

#[derive(Neo4gNode)]
pub struct UserTemplate {
    id: i32,
    name: String,
}

impl From<Node> for User {  // need to generate from macro. Should create a default for each struct as well? Also need to sort Props -> BoltType for query params
    fn from(node: Node) -> Self {
        let mut user = User::new(0, "Testwin".to_string());
        if let Ok(id) = node.get("id") {
            user.id = UserProps::Id(id);
        }
        if let Ok(name) = node.get("name") {
            user.name = UserProps::Name(name);
        }
        user
    }
}

#[derive(Neo4gNode)]
pub struct GroupTemplate {
    id: i32,
    name: String,
    something: String,
}

// pub enum UserProps {
//     Id(i32),
//     Name(String),
// }

// impl Neo4gEntity for User {
//     type Props = UserProps;
//     fn get_entity_type(&self) -> String {
//         "Test".to_string()
//     }

//     fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
//         let mut testsdf: HashMap<String, String> = HashMap::new();
//         testsdf.insert("Test".to_string(), "Test".to_string());
//         ("Test".to_string(), testsdf)
//     }

//     fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
//         let mut testsdf: HashMap<String, String> = HashMap::new();
//         testsdf.insert("Test".to_string(), "Test".to_string());
//         ("Test".to_string(), testsdf)
//     }
// }


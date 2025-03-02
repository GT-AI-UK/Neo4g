use neo4g_macros::Neo4gNode;
use neo4g::neo4g_traits::*;

#[derive(Neo4gNode)]
struct UserTemplate {
    id: i32,
    name: String,
}

#[derive(Neo4gNode)]
struct GroupTemplate {
    id: i32,
    name: String,
    something: String,
}

pub enum EntityWrapper {
    User(User),
    Group(Group),
}

    // impl EntityWrapper {
    //     pub fn inner(self) -> Neo4gEntity {
    //         match self {
    //             EntityWrapper::User(user) => user,
    //             EntityWrapper::Group(group) => group,
    //         }
    //     }
    // }

use std::collections::HashMap;

struct Neo4gBuilder {
    query: String,
    params: HashMap<String, String>,
    node_number: i32,
    relationship_number: i32,
    return_refs: Vec<String>,
    previous_entity: Option<String>,
}

impl Neo4gBuilder {
    fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relationship_number: 0,
            return_refs: Vec::new(),
            previous_entity: None,
        }
    }

    fn create_node<T: Neo4gEntity>(mut self, entity: &T) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        self
    }

    fn match_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.match_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    fn merge_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(name.clone());
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name.clone()));
        self.params.extend(params);
        self
    }

    fn relate_inline<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.relationship_number += 1;
        let name = format!("neo4g_rel{}", self.relationship_number);
        self.previous_entity = Some(name.clone());
        self.query.push_str(&format!("-[neo4g_rel{}:]->", self.relationship_number));//, self.relationship_type));
        self
    }

    // create a relationship macro as well
    // fn relate_inline_with_props(mut self, relationship_type: &str, props: &[HashMap<String, QueryParam>]) -> Self {
    //     self.relationship_number += 1;
    //     self.query.push_str(&format!("-[neo4g_rel{}:{} {{", self.relationship_number, relationship_type));
    //     let vec: Vec<String> = props.iter()
    //         .map(|prop| {
    //             let (key, value) = (prop.key(), value);
    //             self.params.insert(key.to_string(), value);
    //             format!("{}: ${}", key, key)
    //         })
    //         .collect();
    //     self.query.push_str(&format!("{}", vec.join(", ")));
    //     self.query.push_str("}}]->");
    //     self
    // }

    // fn relate_with_node_vars(mut self, ) -> Self {
    //     todo!("");
    // }

    // fn relate_with_node_vars_with_props(mut self, ) -> Self {
    //     todo!("");
    // }

    // fn on_create_set<T: Neo4gEntity>(mut self, props: &[T::Props]) -> Self { // may need to impl T::Props for relationship props in some way or use &[HashMap<String, QueryParam>]
    //     todo!("");
    // }

    // fn on_match_set<T: Neo4gEntity>(mut self, props: &[T::Props]) -> Self { // may need to impl T::Props for relationship props in some way or use &[HashMap<String, QueryParam>]

    // }

    // fn add_to_return(mut self) -> Self {
    //     if let Some(ret_ref) = self.previous_entity {
    //         self.return_refs.push(String::from(ret_ref));
    //     }
    //     self
    // }

    // fn return_objs(mut self) -> Self {
    //     self.query.push(format!("RETURN {}", self.return_refs.join(", ")));
    //     self
    // }

    // fn return_objs_by_refs(mut self, refs: &[&str]) -> Self {
    //     //add to return_refs
    //     //return the query string
    //     todo!("hmmm");
    // }

    fn build(self) -> (String, HashMap<String, String>) {
        (self.query, self.params)
    }

    //async fn run()

    //fn run? (could send query, params, and return values to neo4rs runner?)
}

//fn test<T: Neo4gEntity>(entities: )


fn main() {
    let (query, params) = User::get_node_by(&[UserProps::Name("Test".to_string())]); // Should print: "Generated code for User"
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
}

// use neo4g_macros::{Neo4gNode};
// use std::any::type_name;

// #[derive(Debug)]
// pub enum EntityWrapper {
//     User(User),
// }

// impl EntityWrapper {
//     pub fn inner(self) -> Neo4gEntity {
//         match self {
//             EntityWrapper::User(user) => user,
//         }
//     }
// }
// #[derive(Neo4gNode, Debug, Clone)]
// struct User {
//     id: i32, // this didn't work - user id enum required a string? (UserProps::Id(String???))
//     name: String,
// }

// impl User {
//     fn new() -> EntityWrapper {
//         EntityWrapper::User(User {
//             id: UserProps::Id(0),
//             name: UserProps::Name("test".to_string()),
//         })
//     }
// }

// use std::collections::HashMap;

// struct Neo4gBuilder<E, P> {
//     query: String,
//     params: HashMap<String, P>,
//     node_number: i32,
//     relationship_number: i32,
//     return_refs: Vec<String>,
//     alias_map: HashMap<String, E>,
// }

// impl Neo4gBuilder {
//     fn new() -> Self {
//         Self {
//             query: String::new(),
//             params: HashMap::new(),
//             node_number: 0,
//             relationship_number: 0,
//             return_refs: Vec::new(),
//             alias_map: HashMap::new(),
//         }
//     }

//     //fn q_create<T: Neo4gEntity>(mut self, alias: &str, entity: &T) -> Self {
//     fn q_create(mut self, alias: &str, wrapped_entity: EntityWrapper) -> Self {
//         //let entity_type = entity.get_entity_type();
//         //let entity_props = entity.props();
//         let node_string = String::from("node");
//         let rel_string = String::from("relationsihp");
//         // match entity_type.clone() {
//         //     node_string => {self.node_number += 1;},
//         //     rel_string => {self.relationship_number += 1;}
//         // }
//         println!("{:?}", wrapped_entity.unwrap_as_user().unwrap());
//         if alias.is_empty() {
//             let name = format!("neo4g_node{}", self.node_number);
//         } else {
//             //self.alias_map.insert(alias.to_string(), wrapped_entity);
//             let name = alias.to_owned();
//         }
//         self
//     }

//     fn q_match<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
//         self.node_number += 1;
//         let name = format!("neo4g_node{}", self.node_number);
//         let (query_part, params) = entity.match_by(props);
//         self.query.push_str(&query_part.replace("neo4g_node", &name));
//         self.params.extend(params);
//         self
//     }

//     fn q_merge<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
//         self.node_number += 1;
//         let name = format!("neo4g_node{}", self.node_number);
//         let (query_part, params) = entity.merge_by(props);
//         self.query.push_str(&query_part.replace("neo4g_node", &name));
//         self.params.extend(params);
//         self
//     }

//     // fn relate_inline<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
//     //     self.relationship_number += 1;
//     //     let name = format!("neo4g_rel{}", self.relationship_number);
//     //     self.query.push_str(&format!("-[neo4g_rel{}:{}]->", self.relationship_number));//, self.relationship_type));
//     //     self
//     // }

//     // create a relationship macro as well
//     // fn relate_inline_with_props(mut self, relationship_type: &str, props: &[HashMap<String, QueryParam>]) -> Self {
//     //     self.relationship_number += 1;
//     //     self.query.push_str(&format!("-[neo4g_rel{}:{} {{", self.relationship_number, relationship_type));
//     //     let vec: Vec<String> = props.iter()
//     //         .map(|prop| {
//     //             let (key, value) = (prop.key(), value);
//     //             self.params.insert(key.to_string(), value);
//     //             format!("{}: ${}", key, key)
//     //         })
//     //         .collect();
//     //     self.query.push_str(&format!("{}", vec.join(", ")));
//     //     self.query.push_str("}}]->");
//     //     self
//     // }

//     // fn relate_with_node_vars(mut self, ) -> Self {
//     //     todo!("");
//     // }

//     // fn relate_with_node_vars_with_props(mut self, ) -> Self {
//     //     todo!("");
//     // }

//     // fn on_create_set<T: Neo4gEntity>(mut self, props: &[T::Props]) -> Self { // may need to impl T::Props for relationship props in some way or use &[HashMap<String, QueryParam>]
//     //     todo!("");
//     // }

//     // fn on_match_set<T: Neo4gEntity>(mut self, props: &[T::Props]) -> Self { // may need to impl T::Props for relationship props in some way or use &[HashMap<String, QueryParam>]

//     // }

//     // fn return_objs(mut self) -> Self {
//     //     self.query.push(format!("RETURN {}", self.return_refs.join(", ")));
//     //     self
//     // }

//     fn return_objs_by_refs(mut self, refs: &[&str]) -> Self {
//         //add to return_refs
//         //return the query string
//         todo!("hmmm");
//         self
//     }

//     fn build(self) -> (String, HashMap<String, P>) {
//         (self.query, self.params)
//     }

//     //fn run? (could send query, params, and return values to neo4rs runner?)
// }


// fn main() {
//     let (query, params) = User::get_node_by(&[UserProps::Name("Test".to_string())]); // Should print: "Generated code for User"
//     println!("{}", query);
//     let user = User::new();
//     let test = Neo4gBuilder::new()
//         .q_create("Test", user.clone())
//         .q_match(&user, &[UserProps::Name("Sasd".to_string())])
//         .q_merge(&user, &[UserProps::Name("Sasd".to_string())])
//         .build();
//     println!("{}", test.0)
// }


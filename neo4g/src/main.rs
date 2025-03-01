use neo4g_macros::Neo4g;

#[derive(Neo4g)]
struct User {
    id: i32, // this didn't work - user id enum required a string? (UserProps::Id(String???))
    name: String,
}

enum QueryParam {
    Int(i32),
    String(String),
    Bool(bool),
}

use std::collections::HashMap;

struct Neo4gBuilder {
    query: String,
    params: HashMap<String, QueryParam>,
    node_number: i32,
    relationship_number: i32,

}

impl Neo4gBuilder {
    fn new() -> Self {
        Self {
            query: String::new(),
            params: HashMap::new(),
            node_number: 0,
            relationship_number: 0,
        }
    }

    fn create_node<T: Neo4gEntity>(mut self, entity: &T) -> Self {
        self.node_number += 1;
        self
    }

    fn match_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let (query_part, params) = entity.match_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &format!("neo4g_node{}\n", self.node_number)));
        self.params.extend(params);
        self
    }

    fn merge_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &format!("neo4g_node{}\n", self.node_number)));
        self.params.extend(params);
        self
    }

    fn relate_inline(mut self, relationship_type: &str) -> Self {
        self.relationship_number += 1;
        self.query.push_str(&format!("-[neo4g_rel{}:{}]->", self.relationship_number, relationship_type));
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

    fn build(self) -> (String, HashMap<String, QueryParam>) {
        (self.query, self.params)
    }
}


fn main() {
    let (query, params) = User::get_node_by(&[UserProps::Name("Test".to_string())]); // Should print: "Generated code for User"
    println!("{}", query);
    let user = User {id: 0, name: String::from("Test")};
    let test = Neo4gBuilder::new()
        .match_node(&user, &[UserProps::Name("SDF".to_string())])
        .merge_node(&user, &[UserProps::Name("Sasd".to_string())])
        .build();
    println!("{}", test.0)
}


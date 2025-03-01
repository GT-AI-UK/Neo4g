use neo4g_macros::Neo4gNode;

#[derive(Neo4gNode)]
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
    return_refs: Vec<String>,
    previous_entity: Option<&str>,
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
        self.previous_entity = Some(&name);
        self
    }

    fn match_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(&name);
        let (query_part, params) = entity.match_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name));
        self.params.extend(params);
        self
    }

    fn merge_node<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.node_number += 1;
        let name = format!("neo4g_node{}", self.node_number);
        self.previous_entity = Some(&name);
        let (query_part, params) = entity.merge_by(props);
        self.query.push_str(&query_part.replace("neo4g_node", &name));
        self.params.extend(params);
        self
    }

    fn relate_inline<T: Neo4gEntity>(mut self, entity: &T, props: &[T::Props]) -> Self {
        self.relationship_number += 1;
        let name = format!("neo4g_rel{}", self.relationship_number);
        self.previous_entity = Some(&name);
        self.query.push_str(&format!("-[neo4g_rel{}:{}]->", self.relationship_number));//, self.relationship_type));
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

    fn add_to_return(mut self) -> Self {
        if let Some(ret_ref) = self.previous_entity {
            self.return_refs.push(String::from(ret_ref));
        }
        self
    }

    fn return_objs(mut self) -> Self {
        self.query.push(format!("RETURN {}", self.return_refs.join(", ")));
        self
    }

    fn return_objs_by_refs(mut self, refs: &[&str]) -> Self {
        //add to return_refs
        //return the query string
        todo!("hmmm");
    }

    fn build(self) -> (String, HashMap<String, QueryParam>) {
        (self.query, self.params)
    }

    //fn run? (could send query, params, and return values to neo4rs runner?)
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


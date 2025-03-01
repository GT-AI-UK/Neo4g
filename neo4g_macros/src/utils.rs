use std::collections::HashMap;

// pub trait ToQueryParam {
//     fn to_query_param(&self) -> (&'static str, String);
// }

// /// Converts properties into Cypher query parameters and updates the params map
// pub fn generate_cypher_properties<T: ToQueryParam>(
//     props: &[T],
//     params: &mut HashMap<String, String>,
// ) -> Vec<String> {
//     props
//         .iter()
//         .map(|prop| {
//             let (key, value) = prop.to_query_param();
//             params.insert(key.to_string(), value.clone());
//             format!("{}: ${}", key, key)
//         })
//         .collect()
// }

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
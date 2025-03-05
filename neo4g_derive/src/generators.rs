use quote::quote;
use syn::Ident;
use crate::utils;

pub fn generate_get_node_entity_type() -> proc_macro2::TokenStream {
    quote! {
        fn get_node_entity_type() -> String {
            String::from("node")
        }
    }
}

pub fn generate_get_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        fn get_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
            let mut query = format!("MATCH (neo4g_node:{})", #struct_name_str);
            let mut params = std::collections::HashMap::new();

            if !props.is_empty() {
                let filters: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(key.to_string(), value);
                        format!("neo4g_node.{} = ${}", key, key)
                    })
                    .collect();
                query.push_str(&format!(" WHERE {}", filters.join(" AND ")));
            }
            (query, params)
        }
    }
}

pub fn generate_merge_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
            let mut query = format!("MERGE (neo4g_node:{} {{", #struct_name_str);
            let mut params = std::collections::HashMap::new();

            let props_str: Vec<String> = props
                .iter()
                .map(|prop| {
                    let (key, value) = prop.to_query_param();
                    params.insert(key.to_string(), value);
                    format!("{}: ${}", key, key)
                })
                .collect();

            query.push_str(&props_str.join(", "));
            query.push_str("})");
            (query, params)
        }
    }
}

pub fn generate_from_node_return(new_struct_name: &Ident, new_struct_name_str: &str, fields: &Vec<(&syn::Ident, syn::Type)>) -> proc_macro2::TokenStream {
    let output_struct = quote! {
        let mut output = #new_struct_name::new(9, "test".to_string());
    };
    let database_field_ifs: Vec<_> = fields.iter().map(|(field_ident, _)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), new_struct_name.span());
        let key = syn::LitStr::new(&field_ident.to_string(), new_struct_name.span());
        quote! {
            println!("Getting #key from the thing");
            // if let Ok(field) = node.get(&#key) {
            //     output.#key = field;
            // }
        }
    }).collect();
    
    quote! {
        fn from_node_return(node: neo4rs::Node) -> EntityWrapper {
            #output_struct
            #(#database_field_ifs)*
            EntityWrapper::from(output)
        }
    }
}


pub fn generate_get_relationship_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        fn get_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, QueryParam>) {
            let mut query = format!("MATCH (a)-[neo4g_rel:{}]->(b)", #struct_name_str);
            let mut params = std::collections::HashMap::new();

            if !props.is_empty() {
                let filters: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(key.to_string(), value);
                        format!("neo4g_rel.{} = ${}", key, key)
                    })
                    .collect();
                query.push_str(&format!(" WHERE {}", filters.join(" AND ")));
            }
            query.push_str(" RETURN a, neo4g_rel, b");
            (query, params)
        }
    }
}

pub fn generate_merge_relationship_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        fn merge_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, QueryParam>) {
            let mut query = format!("MATCH (a), (b) MERGE (a)-[neo4g_rel:{} {{", #struct_name_str);
            let mut params = std::collections::HashMap::new();

            let props_str: Vec<String> = props
                .iter()
                .map(|prop| {
                    let (key, value) = prop.to_query_param();
                    params.insert(key.to_string(), value);
                    format!("{}: ${}", key, key)
                })
                .collect();

            query.push_str(&props_str.join(", "));
            query.push_str("}]->(b)");
            (query, params)
        }
    }
}

// use quote::quote;
// use syn::Ident;

// pub fn generate_get_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         fn get_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
//             let mut query = format!("MATCH (neo4g_node:{})", #struct_name_str);
//             let mut params = std::collections::HashMap::new();

//             if !props.is_empty() {
//                 let filters: Vec<String> = props
//                     .iter()
//                     .map(|prop| {
//                         let (key, value) = prop.to_query_param();
//                         params.insert(key.to_string(), value);
//                         format!("neo4g_node.{} = ${}", key, key)
//                     })
//                     .collect();
//                 query.push_str(&format!(" WHERE {}", filters.join(" AND ")));
//             }
//             (query, params)
//         }
//     }
// }

// pub fn generate_merge_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
//             let mut query = format!("MERGE (neo4g_node:{} {{", #struct_name_str);
//             let mut params = std::collections::HashMap::new();

//             let props_str: Vec<String> = props
//                 .iter()
//                 .map(|prop| {
//                     let (key, value) = prop;
//                     params.insert(key.to_string(), value);
//                     format!("{}: ${}", key, key)
//                 })
//                 .collect();

//             query.push_str(&props_str.join(", "));
//             query.push_str("})");
//             (query, params)
//         }
//     }
// }

// pub fn generate_get_relationship_entity_type() -> proc_macro2::TokenStream {
//     quote! {
//         fn get_relationship_entity_type() -> String {
//             String::from("relationship")
//         }
//     }
// }

// pub fn generate_get_relationship_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         fn get_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
//             let mut query = format!("MATCH (a)-[neo4g_rel:{}]->(b)", #struct_name_str);
//             let mut params = std::collections::HashMap::new();

//             if !props.is_empty() {
//                 let filters: Vec<String> = props
//                     .iter()
//                     .map(|prop| {
//                         let (key, value) = prop;
//                         params.insert(key.to_string(), value);
//                         format!("neo4g_rel.{} = ${}", key, key)
//                     })
//                     .collect();
//                 query.push_str(&format!(" WHERE {}", filters.join(" AND ")));
//             }
//             query.push_str(" RETURN a, neo4g_rel, b");
//             (query, params)
//         }
//     }
// }

// pub fn generate_merge_relationship_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         fn merge_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
//             let mut query = format!("MATCH (a), (b) MERGE (a)-[neo4g_rel:{} {{", #struct_name_str);
//             let mut params = std::collections::HashMap::new();

//             let props_str: Vec<String> = props
//                 .iter()
//                 .map(|prop| {
//                     let (key, value) = prop.to_query_param();
//                     params.insert(key.to_string(), value);
//                     format!("{}: ${}", key, key)
//                 })
//                 .collect();

//             query.push_str(&props_str.join(", "));
//             query.push_str("}]->(b)");
//             (query, params)
//         }
//     }
// }

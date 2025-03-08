use quote::quote;
use syn::Ident;

pub fn generate_get_node_entity_type() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_entity_type() -> String {
            String::from("node")
        }
    }
}

// pub fn generate_create_from_self(
//     struct_name: &syn::Ident,
//     struct_name_str: &str,
//     fields: &[(&syn::Ident, syn::Type)]
// ) -> proc_macro2::TokenStream {
//     // Build the property list for the query, e.g. "id: $id, name: $name, ..."
//     let field_properties = fields
//         .iter()
//         .map(|(field_ident, _)| {
//             let field_name = field_ident.to_string();
//             format!("{}: ${}", field_name, field_name)
//         })
//         .collect::<Vec<_>>()
//         .join(", ");
//     let query = format!("CREATE (neo4g_node:{} {{ {} }})", struct_name_str, field_properties);

//     // Generate code for collecting each field’s value into the params vector.
//     let create_params = fields.iter().map(|(field_ident, field_type)| {
//         let field_name = field_ident.to_string();
//         // Check if the field is an Option<T>.
//         let is_optional = if let syn::Type::Path(type_path) = field_type {
//             type_path.qself.is_none() &&
//             type_path.path.segments.len() == 1 &&
//             type_path.path.segments[0].ident == "Option"
//         } else {
//             false
//         };

//         if is_optional {
//             // Extract the inner type T from Option<T>.
//             let inner_type = if let syn::Type::Path(type_path) = field_type {
//                 if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                     &type_path.path.segments[0].arguments
//                 {
//                     if let Some(syn::GenericArgument::Type(inner)) = angle_bracketed.args.first() {
//                         inner
//                     } else {
//                         field_type // fallback if not found
//                     }
//                 } else {
//                     field_type
//                 }
//             } else {
//                 field_type
//             };

//             // If T is a reference type, dereference when converting.
//             let is_inner_ref = matches!(inner_type, syn::Type::Reference(_));

//             if is_inner_ref {
//                 quote! {
//                     if let Some(val) = self.#field_ident() {
//                         params.push((#field_name.to_string(), (*val).into()));
//                     }
//                 }
//             } else {
//                 quote! {
//                     if let Some(val) = self.#field_ident() {
//                         params.push((#field_name.to_string(), val.into()));
//                     }
//                 }
//             }
//         } else {
//             // For non-optional fields, check if the field's type is a reference.
//             let is_ref = matches!(field_type, syn::Type::Reference(_));
//             if is_ref {
//                 quote! {
//                     params.push((#field_name.to_string(), (*self.#field_ident()).into()));
//                 }
//             } else {
//                 quote! {
//                     params.push((#field_name.to_string(), self.#field_ident().into()));
//                 }
//             }
//         }
//     });

//     quote! {
//         pub fn create_from_self(self) -> (String, Vec<(String, BoltType)>) {
//             let query = #query.to_string();
//             let mut params: Vec<(String, BoltType)> = Vec::new();
//             #(#create_params)*
//             (query, params)
//         }
//     }
// }
pub fn generate_create_node_from_self(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn create_node_from_self(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, PropValue>) {
            let mut query = format!("CREATE (neo4g_node:{} {{", #struct_name_str);
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
            query.push_str("})\n");
            (query, params)
        }
    }
}

pub fn generate_get_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, PropValue>) {
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
            query.push_str("\n");
            (query, params)
        }
    }
}

pub fn generate_merge_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, PropValue>) {
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
            query.push_str("})\n");
            (query, params)
        }
    }
}


pub fn generate_get_relation_entity_type() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_entity_type() -> String {
            String::from("relation")
        }
    }
}

pub fn generate_get_relation_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, PropValue>) {
            let mut query = format!("MATCH (a)-[neo4g_rel:{}]->(b)\n", #struct_name_str);
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
            query.push_str("\n");
            (query, params)
        }
    }
}

pub fn generate_merge_relation_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn merge_relation_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, PropValue>) {
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
            query.push_str("}]->(b)\n");
            (query, params)
        }
    }
}

// use quote::quote;
// use syn::Ident;

// pub fn generate_get_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         pub fn get_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
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
//         pub fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
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
//         pub fn get_relationship_entity_type() -> String {
//             String::from("relationship")
//         }
//     }
// }

// pub fn generate_get_relationship_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
//     quote! {
//         pub fn get_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
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
//         pub fn merge_relationship_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, T>) {
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
use quote::quote;
use syn::Ident;

pub fn generate_get_node_entity_type() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_entity_type() -> String {
            String::from("node")
        }
    }
}

pub fn generate_get_node_label(struct_name_str: &str) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_label() -> String {
            String::from(format!("{}", #struct_name_str))
        }
    }
}

pub fn generate_create_node_from_self(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn create_node_from_self(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("(neo4g_node:{} {{", #struct_name_str);
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

pub fn generate_get_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_by(props: &[#props_enum_name]) -> (String, String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("(neo4g_node:{})", #struct_name_str);
            let mut params = std::collections::HashMap::new();
            let mut where_str = String::new();
            if !props.is_empty() {
                let filters: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(key.to_string(), value);
                        format!("neo4g_node.{} = ${}", key, key)
                    })
                    .collect();
                where_str.push_str(&format!("{}", filters.join(" ANDOR ")));
            }
            (query, where_str, params)
        }
    }
}

pub fn generate_merge_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
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


pub fn generate_get_relation_entity_type() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_entity_type() -> String {
            String::from("relation")
        }
    }
}

pub fn generate_get_relation_label(struct_name_str: &str) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_label() -> String {
            String::from(format!("{}", #struct_name_str))
        }
    }
}

pub fn generate_get_relation_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_by(props: &[#props_enum_name]) -> (String, String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("-[neo4g_rel:{}]->", #struct_name_str);
            let mut params = std::collections::HashMap::new();
            let mut where_str = String::new();
            if !props.is_empty() {
                let filters: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(key.to_string(), value);
                        format!("neo4g_rel.{} = ${}", key, key)
                    })
                    .collect();
                where_str.push_str(&format!("{}", filters.join(" ANDOR ")));
            }
            (query, where_str, params)
        }
    }
}

pub fn generate_merge_relation_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn merge_relation_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("-[neo4g_rel:{} {{", #struct_name_str);
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
            query.push_str("}]->");
            (query, params)
        }
    }
}
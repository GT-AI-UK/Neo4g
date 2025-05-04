use quote::quote;
use syn::Ident;

pub fn generate_get_node_entity_type() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_node_entity_type() -> EntityType {
            EntityType::Node
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

pub fn generate_node_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn node_by(alias: &str, props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("({}:{}:AdditionalLabels {{", alias, #struct_name_str);
            let mut params = std::collections::HashMap::new();

            let props_str: Vec<String> = props
                .iter()
                .map(|prop| {
                    let (key, value) = prop.to_query_param();
                    params.insert(format!("{}_{}", alias.to_lowercase(), key.to_string()), value);
                    format!("{}: ${}_{}", key, alias.to_lowercase(), key)
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
        pub fn get_relation_entity_type() -> EntityType {
            EntityType::Relation
        }
    }
}

pub fn generate_set_alias() -> proc_macro2::TokenStream {
    quote! {
        pub fn set_entity_alias(&mut self, alias: &str) {
            self.alias = alias.to_string().to_lowercase();
        }
    }
}

pub fn generate_get_alias() -> proc_macro2::TokenStream {
    quote! {
        pub fn get_entity_alias(&self) -> String {
            self.alias.clone()
        }
    }
}

pub fn generate_get_relation_label(struct_name_str: &str) -> proc_macro2::TokenStream {
    quote! {
        pub fn get_relation_label() -> String {
            String::from(format!("{}", #struct_name_str.to_shouty_snake_case()))
        }
    }
}

pub fn generate_relation_by(struct_name: &Ident, struct_name_str: &str, props_enum_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn relation_by(alias: &str, props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
            let mut query = format!("-[{}:{}", alias, #struct_name_str.to_shouty_snake_case());
            let mut params = std::collections::HashMap::new();
            if !props.is_empty() {
                query.push_str(" {");

                let props_str: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(format!("{}_{}", alias.to_lowercase(), key.to_string()), value);
                        format!("{}: ${}_{}", key, alias.to_lowercase(), key)
                    })
                    .collect();

                query.push_str(&props_str.join(", "));
                query.push_str(" }");
            } else {
                query.push_str("*min_hops..");
            }
            query.push_str("]->");
            (query, params)
        }
    }
}
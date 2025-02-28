use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Neo4g)]
pub fn neo4g_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let props_enum_name = syn::Ident::new(&format!("{}Props", struct_name), struct_name.span());
    let props_enum_variants = fields.iter().map(|field_name| {
        let ident = syn::Ident::new(&capitalize(field_name), struct_name.span());
        quote! { #ident(String) }
    });

    let to_query_param_match_arms = fields.iter().map(|field_name| {
        let ident = syn::Ident::new(&capitalize(field_name), struct_name.span());
        quote! {
            #props_enum_name::#ident(val) => (#field_name, QueryParam::String(val.clone()))
        }
    });

    let get_node_by_fn = generate_get_node_by(&struct_name, &struct_name_str, &props_enum_name);
    let merge_node_by_fn = generate_merge_node_by(&struct_name, &struct_name_str, &props_enum_name);

    let expanded = quote! {
        #[derive(Debug)]
        enum #props_enum_name {
            #(#props_enum_variants),*
        }

        impl #props_enum_name {
            fn to_query_param(&self) -> (&'static str, QueryParam) {
                match self {
                    #(#to_query_param_match_arms),*
                }
            }
        }

        pub trait Neo4gEntity {
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>);
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>);
            type Props;
        }

        impl Neo4gEntity for #struct_name {
            type Props = #props_enum_name;
            
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>) {
                Self::get_node_by(props)
            }
            
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>) {
                Self::merge_node_by(props)
            }
        }
        
        impl #struct_name {
            #get_node_by_fn
            #merge_node_by_fn
        }
    };

    TokenStream::from(expanded)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn generate_get_node_by(struct_name: &syn::Ident, struct_name_str: &str, props_enum_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        fn get_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, QueryParam>) {
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

fn generate_merge_node_by(struct_name: &syn::Ident, struct_name_str: &str, props_enum_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        fn merge_node_by(props: &[#props_enum_name]) -> (String, std::collections::HashMap<String, QueryParam>) {
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

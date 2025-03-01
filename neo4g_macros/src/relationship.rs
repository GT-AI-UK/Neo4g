use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use crate::{generators, utils};

pub fn generate_neo4g_relationship(input: TokenStream) -> TokenStream {
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
        let ident = syn::Ident::new(&utils::capitalize(field_name), struct_name.span());
        quote! { #ident(String) }
    });

    let to_query_param_match_arms = fields.iter().map(|field_name| {
        let ident = syn::Ident::new(&utils::capitalize(field_name), struct_name.span());
        quote! {
            #props_enum_name::#ident(val) => (#field_name, QueryParam::String(val.clone()))
        }
    });

    let get_relationship_by_fn = generators::generate_get_relationship_by(&struct_name, &struct_name_str, &props_enum_name);
    let merge_relationship_by_fn = generators::generate_merge_relationship_by(&struct_name, &struct_name_str, &props_enum_name);

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
        
        pub trait Neo4gRelEntity {
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>);
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>);
            type Props;
        }

        impl Neo4gRelEntity for #struct_name {
            type Props = #props_enum_name;
            
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>) {
                Self::get_relationship_by(props)
            }
            
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, QueryParam>) {
                Self::merge_relationship_by(props)
            }
        }
        
        impl #struct_name {
            #get_relationship_by_fn
            #merge_relationship_by_fn
        }
    };

    TokenStream::from(expanded)
}

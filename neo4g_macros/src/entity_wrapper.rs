use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

pub fn generate_entity_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let unwrap_fn_name = format_ident!("unwrap_as_{}", struct_name);

    let expanded = quote! {

        impl From<#struct_name> for EntityWrapper {
            fn from(entity: #struct_name) -> Self {
                EntityWrapper::#struct_name(entity)
            }
        }
        
        impl EntityWrapper {
            pub fn #unwrap_fn_name(&self) -> Option<&#struct_name> {
                if let EntityWrapper::#struct_name(ref entity) = self {
                    Some(entity)
                } else {
                    None
                }
            }
        }
    };

    TokenStream::from(expanded)
}

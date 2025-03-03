use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

pub fn generate_props_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let unwrap_fn_name = format_ident!("get_{}", enum_name);

    // Ensure the input is an enum.
    let data_enum = match input.data {
        Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "Neo4gPropsWrapper can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut accessors = Vec::new();

    let expanded = quote! {

        impl From<#enum_name> for Neo4gPropsWrapper {
            fn from(props: #enum_name) -> Self {
                Neo4gPropsWrapper::#enum_name(props)
            }
        }
        
        impl Neo4gPropsWrapper {
            pub fn #unwrap_fn_name(&self) -> Option<&#enum_name> {
                if let Neo4gPropsWrapper::#enum_name(ref props) = self {
                    Some(props)
                } else {
                    None
                }
            }
        }
    };

    TokenStream::from(expanded)
}

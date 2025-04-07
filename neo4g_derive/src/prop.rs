use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};
use crate::utils;

pub fn generate_neo4g_prop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Ensure that input.data is an enum, then get the variants.
    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "Neo4gProp can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    //let mut eq_checks = Vec::new();
    let mut from_str_match_arms = Vec::new();
    let mut display_match_arms = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let var_name_str = var_name.to_string();
        let var_name_str_lc = var_name_str.to_lowercase();

        let from_str_arm = quote! {
            #var_name_str_lc => Self::#var_name,
        };
        from_str_match_arms.push(from_str_arm);

        let display_arm = quote! {
            Self::#var_name => #var_name_str,
        };
        display_match_arms.push(display_arm);
        
        // let eq_check = quote! {
        //     (#enum_name::#var_name(_), #enum_name::#var_name(_)) => true,
        // };
        // eq_checks.push(eq_check);
    }

    let gen = quote! {

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Formatter::write_str(f,
                match self {
                    #(#display_match_arms)*
                })
            }
        }

        impl From<String> for #enum_name {
            fn from(value: String) -> #enum_name {
                let v = value.to_lowercase();
                match v.as_ref() {
                    #(#from_str_match_arms)*
                    _ => #enum_name::default()
                }
            }
        }

        impl From<#enum_name> for BoltType {
            fn from(value: #enum_name) -> Self {
                BoltType::String(format!("{}", value).into())
            }
        }

        impl Prop for #enum_name {}
        
        // impl PartialEq for #enum_name {
        //     fn eq(&self, other: &Self) -> bool {
        //         match (self, other) {
        //             #(#eq_checks)*
        //             _ => false,
        //         }
        //     }
        // }
    };

    gen.into()
}
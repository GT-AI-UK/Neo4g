use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};
use crate::utils;

pub fn generate_props_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Ensure that input.data is an enum, then get the variants.
    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "propsWrapper can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let mut accessors = Vec::new();
    let mut _match_arms = Vec::new();
    let mut eq_checks = Vec::new();

        

    let mut to_query_param_match_arms: Vec<_> = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let unwrap_fn_name = format_ident!("get_{}", var_name.to_string().to_lowercase());
        let query_param_match_arm = quote! {
            #enum_name::#var_name(val) => val.to_query_param().clone().into()
        };
        to_query_param_match_arms.push(query_param_match_arm);
        // Generate accessor impls.
        let accessor_tokens = quote! {
            impl From<#var_name> for #enum_name {
                fn from(props: #var_name) -> Self {
                    #enum_name::#var_name(props)
                }
            }
            
            impl #enum_name {
                pub fn #unwrap_fn_name(&self) -> Option<&#var_name> {
                    if let #enum_name::#var_name(ref props) = self {
                        Some(props)
                    } else {
                        None
                    }
                }
            }
        };
        accessors.push(accessor_tokens);

        // (Optional) Generate some match arms for other internal functions.
        let match_arm = quote! {
            #enum_name::#var_name(var) => println!("Matched a {:?}", var),
        };
        _match_arms.push(match_arm);

        // Skip the Nothing variant for label checks.
        if var_name.to_string() == "Nothing" {
            continue;
        }
        // Use the variant name as the label we search for.
        let var_name_str = var_name.to_string();
        let eq_check = quote! {
            (#enum_name::#var_name(_), #enum_name::#var_name(_)) => true,
        };
        eq_checks.push(eq_check);
    }

    // You can keep your existing inner_test function if needed.
    let inner_fn = quote! {
        fn inner_test(&self) -> () {
            let _props = match self {
                #(#_match_arms)*
            };
            println!("{:?}", _props);
        }
    };

    let gen = quote! {
        #(#accessors)*

        impl #enum_name {
            #inner_fn
            pub fn to_query_param(&self) -> (&'static str, BoltType) {
                match self {
                    #(#to_query_param_match_arms),*
                }
            }
            pub fn set_by(alias: &str, props: &[#enum_name]) -> (String, std::collections::HashMap<String, BoltType>) {
                let mut query = String::new();
                let mut params = std::collections::HashMap::new();
    
                let props_str: Vec<String> = props
                    .iter()
                    .map(|prop| {
                        let (key, value) = prop.to_query_param();
                        params.insert(format!("set_{}", key.to_string()), value);
                        format!("{}.{} = $set_{}\n", alias, key, key)
                    })
                    .collect();
    
                query.push_str(&props_str.join(", "));
                (query, params)
            }
        }
        
        impl PartialEq for #enum_name {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    #(#eq_checks)*
                    _ => false,
                }
            }
        }
    };

    gen.into()
}
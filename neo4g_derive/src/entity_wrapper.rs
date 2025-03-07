use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

pub fn generate_entity_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Ensure that input.data is an enum, then get the variants.
    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "EntityWrapper can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let mut accessors = Vec::new();
    let mut _match_arms = Vec::new();
    let mut from_node_checks = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let unwrap_fn_name = format_ident!("get_{}", var_name.to_string().to_lowercase());

        // Generate accessor impls.
        let accessor_tokens = quote! {
            impl From<#var_name> for #enum_name {
                fn from(entity: #var_name) -> Self {
                    #enum_name::#var_name(entity)
                }
            }
            
            impl #enum_name {
                pub fn #unwrap_fn_name(&self) -> Option<&#var_name> {
                    if let #enum_name::#var_name(ref entity) = self {
                        Some(entity)
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
        let check = quote! {
            if labels.contains(&#var_name_str) {
                println!("labels contains {}", #var_name_str);
                return #enum_name::#var_name(#var_name::from(node));
            }
        };
        from_node_checks.push(check);
    }

    // You can keep your existing inner_test function if needed.
    let inner_fn = quote! {
        fn inner_test(&self) -> () {
            let _entity = match self {
                #(#_match_arms)*
            };
            println!("{:?}", _entity);
        }
    };

    // Generate the from_node function.
    let from_node_fn = quote! {
        pub fn from_node(node: Node) -> Self {
            let labels = node.labels();
            #(#from_node_checks)*
            // Fallback: if no label matched, return the Nothing variant.
            #enum_name::Nothing(Nothing::new(true))
        }
    };

    let gen = quote! {
        #(#accessors)*

        impl #enum_name {
            #inner_fn
            #from_node_fn
        }
        
        impl PartialEq for #enum_name {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (#enum_name::User(_), #enum_name::User(_)) => true,
                    (#enum_name::Group(_), #enum_name::Group(_)) => true,
                    _ => false,
                }
            }
        }
    };

    gen.into()
}
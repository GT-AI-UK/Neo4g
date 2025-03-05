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
                "Neo4gPropsWrapper can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let mut accessors = Vec::new();
    let mut match_arms = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let unwrap_fn_name = format_ident!("get_{}", var_name);

        let accessor_tokens = quote! {
            impl From<#var_name> for #enum_name {
                fn from(entity: #var_name) -> Self {
                    #enum_name::#var_name(entity)
                }
            }
            
            impl #enum_name {
                fn #unwrap_fn_name(&self) -> Option<&#var_name> {
                    if let #enum_name::#var_name(ref entity) = self {
                        Some(entity)
                    } else {
                        None
                    }
                }
            }
        };
        accessors.push(accessor_tokens);

        let match_arm = quote! {
            #enum_name::#var_name(var) => println!("Matched a {:?}", var),
        };
        match_arms.push(match_arm); // use two tuples:
        //(User, Group, etc.)
        //None, Some(1), None, etc.
        // iterate over tuples until a Some() is reached, get the associated value from the other
        // use if let? to update mutable tuples?
    }

    let inner_fn = quote! {
        fn inner_test(&self) -> () {// #enum_name {
            let entity = match self {
                #(#match_arms)*
            };
            println!("{:?}", entity);
            //#enum_name::from(entity)
        }
    };


    let gen = quote! {
        #(#accessors)*
        impl #enum_name {
            #inner_fn
        }
        impl PartialEq for EntityWrapper {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (EntityWrapper::User(_), EntityWrapper::User(_)) => true,
                    (EntityWrapper::Group(_), EntityWrapper::Group(_)) => true,
                    _ => false,
                }
            }
        }
    };
    gen.into()
}

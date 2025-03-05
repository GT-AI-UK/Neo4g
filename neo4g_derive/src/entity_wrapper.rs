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

        //(User, Group, etc.)
        //None, Some(1), None, etc.
        // iterate over tuples until a Some() is reached, get the associated value from the other
        // use if let? to update mutable tuples?
    }

    let tuple_type = {
        let types = data_enum.variants.iter().map(|variant| {
            // Assumes each variant is a newtype variant
            let field = variant.fields.iter().next().expect("Expected a single field");
            let ty = &field.ty;
            quote! { Option<& #ty> }
        });
        quote! { ( #(#types),* ) }
    };

    let match_arms = data_enum.variants.iter().map(|current_variant| {
        let current_ident = &current_variant.ident;
        // Build the tuple literal by iterating over all variants.
        let tuple_expr = {
            let fields = data_enum.variants.iter().map(|v| {
                if v.ident == *current_ident {
                    // For the matched variant, produce Some(value)
                    quote! { Some(value) }
                } else {
                    // For the others, produce None
                    quote! { None }
                }
            });
            quote! { ( #(#fields),* ) }
        };
    
        quote! {
            #enum_name::#current_ident(ref value) => #tuple_expr
        }
    });
    
    let tuple_stuff = quote! {
        pub fn as_tuple(&self) -> #tuple_type {
            match self {
                #(#match_arms),*
            }
        }
    };

    // let inner_fn = quote! {
    //     fn inner_test(&self) -> #enum_name { //forget having inner test. Run the query here - use as_tuple to destructure:
    //         // while let result
    //         // return node = destructured tuple of that type.
    //         // destructured_var::from(node) - use node.get()
    //         // return type of the line above is an EntityWrapper.
    //         // append to entity wrapper vec
    //         // return vec. caller of querybuilder can destructure :D
    //         let entity = match self {
    //             #(#match_arms)*
    //         };
    //         println!("{:?}", entity);
    //         #enum_name::from(entity)
    //     }
    // };


    let gen = quote! {
        #(#accessors)*
        impl #enum_name {
            #tuple_stuff
            pub fn boring() -> () {};
           // #inner_fn
        }
    };
    gen.into()
}

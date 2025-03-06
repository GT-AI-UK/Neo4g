use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
//use neo4g_traits::*;
use crate::{generators, utils};

pub fn generate_neo4g_node(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();

    // Collect each field's identifier and type from the template struct.
    let fields: Vec<(&syn::Ident, syn::Type)> = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| (f.ident.as_ref().unwrap(), f.ty.clone()))
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Generate the base name by removing the "Template" suffix (if present).
    let base_name = struct_name_str.trim_end_matches("Template");
    // Generated Props enum (e.g. UserProps).
    let props_enum_name = syn::Ident::new(&format!("{}Props", base_name), struct_name.span());

    // Generate enum variants that hold the actual field types.
    let props_enum_variants: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
        quote! { #variant(#field_type) }
    }).collect();

    let variant_idents: Vec<_> = fields.iter().map(|(field_ident, _)| {
        syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span())
    }).collect();

    let enum_definition = quote! {
        pub enum #props_enum_name {
            #(#props_enum_variants),*
        }
    };
    
    let into_impl = quote! {
        impl Into<BoltType> for #props_enum_name {
            fn into(self) -> BoltType {
                match self {
                    #(
                        #props_enum_name::#variant_idents(val) => val.into(),
                    )*
                }
            }
        }
    };
    

    // Generate match arms for converting a variant’s inner value to a query parameter.
    let to_query_param_match_arms: Vec<_> = fields.iter().map(|(field_ident, _)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
        let key = syn::LitStr::new(&field_ident.to_string(), struct_name.span());
        quote! {
            #props_enum_name::#variant(val) => (#key, val, struct_name.span())
        }
    }).collect();

    // Generate accessor methods for the Props enum.
    // For non-optional fields, return &T; for Option<T>, return Option<&T>.
    let props_accessor_methods: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());

        // Check if the field type is Option<T>.
        let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
            if type_path.qself.is_none()
                && type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option"
            {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) =
                    &type_path.path.segments[0].arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) =
                        angle_bracketed.args.first()
                    {
                        Some(inner_ty)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(inner_type) = maybe_inner_type {
            quote! {
                pub fn #method_ident(&self) -> Option<&#inner_type> {
                    match self {
                        Self::#variant(ref opt) => opt.as_ref(),
                        _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
                    }
                }
            }
        } else {
            quote! {
                pub fn #method_ident(&self) -> &#field_type {
                    match self {
                        Self::#variant(ref val) => val,
                        _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
                    }
                }
            }
        }
    }).collect();

    // Generate the new struct name by removing "Template" from the original struct name.
    let new_struct_name = syn::Ident::new(base_name, struct_name.span());
    let new_struct_name_str = new_struct_name.to_string();

    // Generate fields for the new struct: same field names, but type is the Props enum.
    let new_struct_fields: Vec<_> = fields.iter().map(|(field_ident, _)| {
        quote! {
            pub #field_ident: #props_enum_name
        }
    }).collect();

    // Generate the constructor parameters (with the original types).
    let constructor_params: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        quote! {
            #field_ident: #field_type
        }
    }).collect();

    // Generate a list of field identifiers for forwarding.
    let constructor_args: Vec<_> = fields.iter().map(|(field_ident, _)| {
        quote! { #field_ident }
    }).collect();

    // Generate the constructor body that wraps each field value in the corresponding Props variant.
    let constructor_body: Vec<_> = fields.iter().map(|(field_ident, _)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
        quote! {
            #field_ident: #props_enum_name::#variant(#field_ident)
        }
    }).collect();

    // Generate accessor methods for the new struct.
    // These delegate to the corresponding Props accessor methods.
    let struct_accessor_methods: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
        // Check if field_type is Option<T>
        let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
            if type_path.qself.is_none()
                && type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option"
            {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) =
                    &type_path.path.segments[0].arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) =
                        angle_bracketed.args.first()
                    {
                        Some(inner_ty)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
    
        if let Some(inner_ty) = maybe_inner_type {
            // Field is optional: return Option<&InnerType>
            quote! {
                pub fn #method_ident(&self) -> Option<&#inner_ty> {
                    self.#field_ident.#method_ident()
                }
            }
        } else {
            // Field is required: return &FieldType
            quote! {
                pub fn #method_ident(&self) -> &#field_type {
                    self.#field_ident.#method_ident()
                }
            }
        }
    }).collect();
    
    // Generate a constructor method for the generated struct.
    let generated_constructor = quote! {
        pub fn new( #(#constructor_params),* ) -> Self {
            Self {
                #(#constructor_body),*
            }
        }
    };

    // Generate the new() method for the template struct that forwards to the generated struct's new().
    let template_new_method = quote! {
        impl #struct_name {
            pub fn new( #(#constructor_params),* ) -> #new_struct_name {
                #new_struct_name::new( #(#constructor_args),* )
            }
        }
    };

    // Generate query functions using the generated Props enum.
    let get_node_entity_type_fn = generators::generate_get_node_entity_type();
    let get_node_by_fn = generators::generate_get_node_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
    let merge_node_by_fn = generators::generate_merge_node_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
    let from_node_return_fn = generators::generate_from_node_return(&new_struct_name, &new_struct_name_str, &fields);

    // Assemble the final output.
    let expanded = quote! {
        // Generated Props enum.
        #[derive(Debug, Clone)]
        pub enum #props_enum_name {
            #(#props_enum_variants),*
        }

        impl #props_enum_name {
            /// Converts a Props variant to a key and its stringified value.
            pub fn to_query_param(&self) -> (&'static str, T) {
                match self {
                    #(#to_query_param_match_arms),*
                }
            }

            // Accessor methods for the Props enum.
            #(#props_accessor_methods)*
        }

        // Generated new struct (e.g., `User` from `UserTemplate`) whose fields are wrapped in the Props enum.
        #[derive(Debug, Clone)]
        pub struct #new_struct_name {
            #(#new_struct_fields),*
        }

        // Implement the Neo4gEntity trait from neo4g_traits.
        impl Neo4gEntity for #new_struct_name {
            type Props = #props_enum_name;

            fn get_entity_type(&self) -> String {
                Self::get_node_entity_type()
            }
            
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
                Self::get_node_by(props)
            }
            
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>) {
                Self::merge_node_by(props)
            }

            fn from_node(node: neo4rs::Node) -> EntityWrapper {
                Self::from_node_return(node)
            }
        }
        
        impl #new_struct_name {
            #generated_constructor
            #get_node_entity_type_fn
            #get_node_by_fn
            #merge_node_by_fn
            #from_node_return_fn
            #(#struct_accessor_methods)*
            
        }

        // Constructor for the generated struct.
        

        // New() method for the template struct that forwards to the generated struct's new().
        #template_new_method

        // Accessor methods for the generated struct.
        //#struct_impl

        #into_impl
    };

    TokenStream::from(expanded)
}

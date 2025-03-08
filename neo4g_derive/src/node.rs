use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};
//use neo4g_traits::*;
use crate::{generators, utils};

pub fn generate_neo4g_node(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();
    // Generate the new struct name by removing "Template" from the original struct name.
    // Generate the base name by removing the "Template" suffix (if present).
    let base_name = struct_name_str.trim_end_matches("Template");
    let new_struct_name = syn::Ident::new(base_name, struct_name.span());
    let new_struct_name_str = new_struct_name.to_string();

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

    
    // Generated Props enum (e.g. UserProps).
    let props_enum_name = syn::Ident::new(&format!("{}Props", base_name), struct_name.span());

    // Generate enum variants that hold the actual field types.
    let props_enum_variants: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
        quote! { #variant(#field_type) }
    }).collect();

    let create_node_params: Vec<_> = fields.iter().map(|(field_ident, _)| {
        let field_name = field_ident.to_string();
        // Create a literal string for the field name.
        let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
        // We assume the accessor method has the same name as the field.
        let access_method_ident = syn::Ident::new(&field_name, field_ident.span());
        quote! {
            (#field_name_lit.to_string(), BoltType::from(self.#access_method_ident().clone()))
        }
    }).collect();
    
    let create_query_params: Vec<_> = fields.iter().map(|(field_ident, _)| {
        let field_name = field_ident.to_string();
        let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
        quote! {
            format!("{}: ${}", #field_name_lit, #field_name_lit)
        }
    }).collect();
    
    let create_node_from_self_fn = quote! {
        pub fn create_node_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
            let keys: Vec<String> = vec![ #(#create_query_params),* ];
            let query = format!("CREATE (neo4g_node:{} {{{}}})\n", #new_struct_name_str, keys.join(", "));
            let params_map: std::collections::HashMap<String, BoltType> = std::collections::HashMap::from([
                #(#create_node_params),*
            ]);
            (query, params_map)
        }
    };

    // Generate match arms for converting a variant’s inner value to a query parameter.
    let to_query_param_match_arms: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
        let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
        let key = syn::LitStr::new(&field_ident.to_string(), struct_name.span());

        // Convert the field type into a string so we can match on it.
        let field_type_str = field_type.to_token_stream().to_string();

        // Determine the correct conversion for each type.
        let conversion = if field_type_str == "String" {
            quote! {
                BoltType::String(BoltString::from(val.clone()))
            }
        } else if ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", ].contains(&field_type_str.as_str()) {
            quote! {
                BoltType::Integer(BoltInteger::from(*val))
            }
        } else {
            // Fallback: convert the value to a string and wrap it in a BoltType::String.
            quote! {
                BoltType::String(BoltString::from(val.to_string()))
            }
        };

        quote! {
            #props_enum_name::#variant(val) => (#key, #conversion)
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
        impl #new_struct_name {
            pub fn new( #(#constructor_params),* ) -> Self {
                Self {
                    #(#constructor_body),*
                }
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

    // Generate the impl block for the new struct with accessor methods
    let struct_impl = quote! {
        impl #new_struct_name {
            #(#struct_accessor_methods)*
        }
    };

        // Generate field initializers for the From<Node> impl.
        let field_inits: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
            // Create the variant name (capitalized) for the Props enum.
            let variant = syn::Ident::new(&utils::capitalize(&field_ident.to_string()), struct_name.span());
            // Create a literal string key for node lookup.
            let key = syn::LitStr::new(&field_ident.to_string(), struct_name.span());
            // Convert the field type into a string for matching.
            let field_type_str = field_type.to_token_stream().to_string();

            // Generate the extraction expression based on the type.
            let extraction = if field_type_str == "String" {
            // For String: simply extract from the node.
            quote! {
                node.get(#key).unwrap_or_default()
            }
            } else if ["i8", "i16", "i32", "i64", "i128",
                    "u8", "u16", "u32", "u64", "u128"]
                .contains(&field_type_str.as_str())
            {
            // For integer types: assume node.get returns a u64, then cast.
            quote! {
                {
                    let tmp: u64 = node.get(#key).unwrap_or_default();
                    tmp as #field_type
                }
            }
            } else if field_type_str == "bool" {
                // For bool: extract the value (bool::default() is false).
                quote! {
                    node.get(#key).unwrap_or_default()
                }
            } else if field_type_str == "f32" || field_type_str == "f64" {
            // For floating point types: assume node.get returns a f64, then cast.
            quote! {
                {
                    let tmp: f64 = node.get(#key).unwrap_or_default();
                    tmp as #field_type
                }
            }
            } else {
                // Fallback: simply extract the value.
                quote! {
                    node.get(#key).unwrap_or_default()
                }
            };

            // Wrap the extracted value in the corresponding Props enum variant.
            quote! {
                #field_ident: #props_enum_name::#variant(#extraction)
            }
        }).collect();

        // Generate the complete From<Node> implementation for the struct.
        let from_impl = quote! {
            impl From<Node> for #new_struct_name {
                fn from(node: Node) -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }
            }
        };

        // Generate query functions using the generated Props enum.
        let get_node_entity_type_fn = generators::generate_get_node_entity_type();
        let get_node_by_fn = generators::generate_get_node_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
        let merge_node_by_fn = generators::generate_merge_node_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
        //let create_node_from_self_fn = generators::generate_create_node_from_self(&new_struct_name, &new_struct_name_str, &props_enum_name);

    // Assemble the final output.
    let expanded = quote! {
        // Generated Props enum.
        #[derive(Debug, Clone)]
        pub enum #props_enum_name {
            #(#props_enum_variants),*
        }

        impl #props_enum_name {
            /// Converts a Props variant to a key and its stringified value.
            pub fn to_query_param(&self) -> (&'static str, BoltType) {
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
            
            fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
                Self::get_node_by(props)
            }
            
            fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
                Self::merge_node_by(props)
            }

            fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
                self.create_node_from_self()
            }
        }
        
        impl #new_struct_name {
            #get_node_entity_type_fn
            #get_node_by_fn
            #merge_node_by_fn
            #create_node_from_self_fn
        }

        // Constructor for the generated struct.
        #generated_constructor

        // New() method for the template struct that forwards to the generated struct's new().
        #template_new_method

        #from_impl

        // Accessor methods for the generated struct.
        #struct_impl
    };

    TokenStream::from(expanded)
}
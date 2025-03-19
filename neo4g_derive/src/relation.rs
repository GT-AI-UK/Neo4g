use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};
//use neo4g_traits::*;
use crate::{generators, utils};
use heck::ToPascalCase;

pub fn generate_neo4g_relation(input: TokenStream) -> TokenStream {
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
    let all_fields_full: Vec<&syn::Field> = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields.named.iter().collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Helper to check if a field has the ignore attribute.
    fn should_ignore_field(field: &syn::Field) -> bool {
        field.attrs.iter().any(|attr| attr.path().is_ident("not_query_param"))
    }

    // Generated Props enum (e.g. UserProps).
    let props_enum_name = syn::Ident::new(&format!("{}Props", base_name), struct_name.span());

    // Generate enum variants that hold the actual field types.
    let props_enum_variants: Vec<_> = all_fields_full.iter().map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
            quote! { #variant(#field_type) }
        } else {
            quote! {}
        }
    }).collect();

    let create_relation_params: Vec<_> = all_fields_full.iter().map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string();
            // Create a literal string for the field name.
            let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
            // We assume the accessor method has the same name as the field.
            let access_method_ident = syn::Ident::new(&field_name, field_ident.span());
            quote! {
                (#field_name_lit.to_string(), BoltType::from(self.#access_method_ident().clone()))
            }
        } else {
            quote! {}
        }
    }).collect();
    
    let create_query_params: Vec<_> = all_fields_full.iter().map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();

            let field_name = field_ident.to_string();
            let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
            
            quote! {
                format!("{}: ${}", #field_name_lit, #field_name_lit)
            }
        } else {
            quote! {}
        }
    }).collect();
    
    let create_relation_from_self_fn = quote! {
        pub fn create_relation_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
            let keys: Vec<String> = vec![ #(#create_query_params),* ];
            let query = format!("-[neo4g_relation:{} {{{}}}]->", #new_struct_name_str, keys.join(", "));
            let params_map: std::collections::HashMap<String, BoltType> = std::collections::HashMap::from([
                #(#create_relation_params),*
            ]);
            (query, params_map)
        }
    };

    // Generate match arms for converting a variant’s inner value to a query parameter.
    let to_query_param_match_arms: Vec<_> = all_fields_full.iter().map(|(field)| {
        let field_ident = field.ident.as_ref().unwrap();
        let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
        let key_lit = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
        
        if should_ignore_field(field) {
            // For ignored fields, provide a match arm that essentially does nothing.
            quote! {}
        } else {
            // For normal fields, return the key and the value.
            quote! {
                #props_enum_name::#variant(val) => (#key_lit, val.clone().into())
            }
        }
    }).collect();

    // Generate accessor methods for the Props enum.
    // For non-optional fields, return &T; for Option<T>, return Option<&T>.
    let props_accessor_methods: Vec<_> = all_fields_full.iter().map(|field| {
        if should_ignore_field(field) {
            quote! {}
        } else {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());

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
        }
    }).collect();

    // let props_impl = quote! {
    //     impl #new_struct_name {
    //         /// Converts a Props variant to a key and its stringified value.
    //         pub fn to_query_param(&self) -> (&'static str, BoltType) {
    //             match self {
    //                 #(#to_query_param_match_arms),*
    //             }
    //         }
    
    //         // Accessor methods for the Props enum.
    //         #(#props_accessor_methods)*
    //     }
    // };

    // Generate fields for the new struct: same field names, but type is the Props enum.
    let new_struct_fields: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
    
        if should_ignore_field(field) {
            quote! {
                pub #field_ident: #field_ty
            }
        } else {
            quote! {
                pub #field_ident: #props_enum_name
            }
        }
    }).collect();
    
    // 2. Generate the constructor parameters (using the original types for all fields).
    let constructor_params: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_type_str = field_ty.to_token_stream().to_string();
        //could/SHOULD propbably convert String props to take &str args in constructors
        quote! {
            #field_ident: #field_ty
        }
    }).collect();
    
    // 3. Generate a list of field identifiers for forwarding.
    let constructor_args: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        quote! { #field_ident }
    }).collect();
    
    // 4. Generate the constructor body. For non-ignored fields, we wrap the value in the
    // corresponding Props enum variant; for ignored fields, we simply pass the value through.
    let constructor_body: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        if should_ignore_field(field) {
            quote! {
                #field_ident: #field_ident
            }
        } else {
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
            quote! {
                #field_ident: #props_enum_name::#variant(#field_ident)
            }
        }
    }).collect();

    let template_constructor_body: Vec<_> = all_fields_full.iter().map(|field| {field.ident.as_ref().unwrap()}).collect();

    let default_body: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        if should_ignore_field(field) {
            quote! {
                #field_ident: Default::default()
            }
        } else {
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
            quote! {
                #field_ident: #props_enum_name::#variant(Default::default())
            }
        }
    }).collect();

    // Generate accessor methods for the new struct.
    // These delegate to the corresponding Props accessor methods.
let struct_accessor_methods: Vec<_> = all_fields_full.iter().map(|field| {
        if should_ignore_field(field) {
            quote! {}
        } else {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
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
        }
    }).collect();
    
    // Generate a constructor method for the generated struct.
    let generated_constructor = quote! {
        impl #new_struct_name {
            pub fn new( #(#constructor_params),* ) -> Self {
                Self {
                    alias: String::new(),
                    #(#constructor_body),*
                }
            }
        }
    };

    let generated_default = quote! {
        impl Default for #new_struct_name {
            fn default() -> Self {
                Self {
                    alias: String::new(),
                    #(#default_body),*
                }
            }
        }
    };

    // Generate the new() method for the template struct that forwards to the generated struct's new().
    let template_new_method = quote! {
        impl #struct_name {
            pub fn new( #(#constructor_params),* ) -> Self {
                Self {
                    #(#template_constructor_body),*
                }
            }
        }
    };

    // Generate the impl block for the new struct with accessor methods
    let struct_impl = quote! {
        impl #new_struct_name {
            #(#struct_accessor_methods)*
        }
    };

        // Generate field initializers for the From<relation> impl.
        let field_inits: Vec<_> = all_fields_full.iter().map(|(field)| {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = field.ty.clone();
        
            if should_ignore_field(field) {
                // For ignored fields, we just use their default value.
                quote! {
                    #field_ident: Default::default()
                }
            } else {
                // For fields that go into queries, wrap them in the Props enum.
                let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
                let key = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
                let field_type_str = field_type.to_token_stream().to_string();
        
                // Generate extraction expression based on the type.
                let extraction = if field_type_str == "String" {
                    quote! {
                        relation.get(#key).unwrap_or_default()
                    }
                } else if ["i8", "i16", "i32", "i64", "i128",
                          "u8", "u16", "u32", "u64", "u128"]
                    .contains(&field_type_str.as_str())
                {
                    quote! {
                        {
                            let tmp: u64 = relation.get(#key).unwrap_or_default();
                            tmp as #field_type
                        }
                    }
                } else if field_type_str == "bool" {
                    quote! {
                        relation.get(#key).unwrap_or_default()
                    }
                } else if field_type_str == "f32" || field_type_str == "f64" {
                    quote! {
                        {
                            let tmp: f64 = relation.get(#key).unwrap_or_default();
                            tmp as #field_type
                        }
                    }
                } else {
                    quote! {
                        relation.get(#key).unwrap_or_default()
                    }
                };
        
                quote! {
                    #field_ident: #props_enum_name::#variant(#extraction)
                }
            }
        }).collect();

        // Generate the complete From<relation> implementation for the struct.
        let from_impl = quote! {
            impl From<Relation> for #new_struct_name {
                fn from(relation: Relation) -> Self {
                    Self {
                        alias: String::new(),
                        #(#field_inits),*
                    }
                }
            }
        };

        let to_template_fields: Vec<_> = all_fields_full.iter().map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            if should_ignore_field(field) {
                quote! {
                    #field_ident: new.#field_ident.clone()
                }       
            } else {
                quote! {
                    #field_ident: new.#field_ident().clone()
                } 
            }
        }).collect();

        let to_template_impl = quote! {
            impl From<#new_struct_name> for #struct_name {
                fn from(new: #new_struct_name) -> Self {
                    Self {
                        #(#to_template_fields),*
                    }
                }
            }
        };

        let from_template_fields: Vec<_> = all_fields_full.iter().map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
            if should_ignore_field(field) {
                quote! {
                    #field_ident: template.#field_ident.clone()
                }       
            } else {
                quote! {
                    #field_ident: #props_enum_name::#variant(template.#field_ident.clone())
                } 
            }
        }).collect();

        let from_template_impl = quote! {
            impl From<#struct_name> for #new_struct_name {
                fn from(template: #struct_name) -> Self {
                    Self {
                        alias: String::new(),
                        #(#from_template_fields),*
                    }
                }
            }
        };

        let silly_from_impl = quote! {
            impl From<Node> for #new_struct_name {
                fn from(relation: Node) -> Self {
                    Self {
                        alias: String::new(),
                        #(#field_inits),*
                    }
                }
            }
        };

        // Generate query functions using the generated Props enum.
        let get_relation_entity_type_fn = generators::generate_get_relation_entity_type();
        //let get_relation_by_fn = generators::generate_get_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
        let relation_by_fn = generators::generate_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
        let get_relation_label_fn = generators::generate_get_relation_label(&new_struct_name_str);
        let set_alias_fn = generators::generate_set_alias();
        let get_alias_fn = generators::generate_get_alias();

    // Assemble the final output.
    let expanded = quote! {
        // Generated Props enum.
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(untagged)]
        pub enum #props_enum_name {
            #(#props_enum_variants),*
        }

        impl QueryParam for #props_enum_name {
            fn to_query_param(&self) -> (&'static str, BoltType) {
                match self {
                    #(#to_query_param_match_arms),*
                }
            }
        }

        impl #props_enum_name {
            #(#props_accessor_methods)*
        }

        // Generated new struct (e.g., `User` from `UserTemplate`) whose fields are wrapped in the Props enum.
        #[derive(Serialize, Deserialize, Debug, Clone)]
        
        pub struct #new_struct_name {
            alias: String,
            #(#new_struct_fields),*
        }

        // Implement the Neo4gEntity trait from neo4g_traits.
        impl Neo4gEntity for #new_struct_name {
            type Props = #props_enum_name;

            fn get_entity_type(&self) -> String {
                Self::get_relation_entity_type()
            }

            fn get_label(&self) -> String {
                Self::get_relation_label()
            }
            
            // fn match_by(&self, props: &[Self::Props]) -> (String, String, std::collections::HashMap<String, BoltType>) {
            //     Self::get_relation_by(props)
            // }
            
            fn set_alias(&mut self, alias: &str) {
                self.set_entity_alias(alias);
            }

            fn get_alias(&self) -> String {
                self.get_entity_alias()
            }

            fn entity_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
                Self::relation_by(props)
            }

            fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
                self.create_relation_from_self()
            }
        }
        
        impl #new_struct_name {
            #get_relation_entity_type_fn
            //#get_relation_by_fn
            #relation_by_fn
            #create_relation_from_self_fn
            #get_relation_label_fn
            #set_alias_fn
            #get_alias_fn
        }

        // Constructor for the generated struct.
        #generated_constructor
        #generated_default

        // New() method for the template struct that forwards to the generated struct's new().
        #template_new_method

        #from_impl
        #silly_from_impl // could have a different trait to handle the from impl maybe? can functions take two traits?

        #to_template_impl
        #from_template_impl
        // Accessor methods for the generated struct.
        #struct_impl
    };

    TokenStream::from(expanded)
}

// use proc_macro::TokenStream;
// use quote::{quote, ToTokens};
// use syn::{parse_macro_input, DeriveInput, Data, Fields};
// //use neo4g_traits::*;
// use crate::{generators, utils};

// pub fn generate_neo4g_relation(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree.
//     let input = parse_macro_input!(input as DeriveInput);
//     let struct_name = &input.ident;
//     let struct_name_str = struct_name.to_string();
//     // Generate the new struct name by removing "Template" from the original struct name.
//     // Generate the base name by removing the "Template" suffix (if present).
//     let base_name = struct_name_str.trim_end_matches("Template");
//     let new_struct_name = syn::Ident::new(base_name, struct_name.span());
//     let new_struct_name_str = new_struct_name.to_string();

//     // Collect each field's identifier and type from the template struct.
//     let all_fields_full: Vec<&syn::Field> = if let Data::Struct(data_struct) = &input.data {
//         if let Fields::Named(fields) = &data_struct.fields {
//             fields.named.iter().collect()
//         } else {
//             vec![]
//         }
//     } else {
//         vec![]
//     };

//     // Helper to check if a field has the ignore attribute.
//     fn should_ignore_field(field: &syn::Field) -> bool {
//         field.attrs.iter().any(|attr| attr.path().is_ident("not_query_param"))
//     }
    
//     // Generated Props enum (e.g. UserProps).
//     let props_enum_name = syn::Ident::new(&format!("{}Props", base_name), struct_name.span());

//     // Generate enum variants that hold the actual field types.
//     let props_enum_variants: Vec<_> = all_fields_full.iter().map(|field| {
//         if !should_ignore_field(field) {
//             let field_ident = field.ident.as_ref().unwrap();
//             let field_type = &field.ty;
//             let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
//             quote! { #variant(#field_type) }
//         } else {
//             quote! {}
//         }
//     }).collect();

//     let create_relation_params: Vec<_> = all_fields_full.iter().map(|field| {
//         if !should_ignore_field(field) {
//             let field_ident = field.ident.as_ref().unwrap();
//             let field_name = field_ident.to_string();
//             // Create a literal string for the field name.
//             let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
//             // We assume the accessor method has the same name as the field.
//             let access_method_ident = syn::Ident::new(&field_name, field_ident.span());
//             quote! {
//                 (#field_name_lit.to_string(), BoltType::from(self.#access_method_ident().clone()))
//             }
//         } else {
//             quote! {}
//         }
//     }).collect();
    
//     let create_query_params: Vec<_> = all_fields_full.iter().map(|field| {
//         if !should_ignore_field(field) {
//             let field_ident = field.ident.as_ref().unwrap();

//             let field_name = field_ident.to_string();
//             let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
            
//             quote! {
//                 format!("{}: ${}", #field_name_lit, #field_name_lit)
//             }
//         } else {
//             quote! {}
//         }
//     }).collect();
    
    

//     // Generate match arms for converting a variant’s inner value to a query parameter.
//     let to_query_param_match_arms: Vec<_> = all_fields_full.iter().map(|(field)| {
//         let field_ident = field.ident.as_ref().unwrap();
//         let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
//         let key_lit = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
        
//         if should_ignore_field(field) {
//             // For ignored fields, provide a match arm that essentially does nothing.
//             quote! {}
//         } else {
//             // For normal fields, return the key and the value.
//             quote! {
//                 #props_enum_name::#variant(val) => (#key_lit, val.clone().into())
//             }
//         }
//     }).collect();

//     // Generate accessor methods for the Props enum.
//     // For non-optional fields, return &T; for Option<T>, return Option<&T>.
//     let props_accessor_methods: Vec<_> = all_fields_full.iter().map(|field| {
//         if should_ignore_field(field) {
//             quote! {}
//         } else {
//             let field_ident = field.ident.as_ref().unwrap();
//             let field_type = &field.ty;
//             let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
//             let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());

//             // Check if the field type is Option<T>.
//             let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
//                 if type_path.qself.is_none()
//                     && type_path.path.segments.len() == 1
//                     && type_path.path.segments[0].ident == "Option"
//                 {
//                     if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                         &type_path.path.segments[0].arguments
//                     {
//                         if let Some(syn::GenericArgument::Type(inner_ty)) =
//                             angle_bracketed.args.first()
//                         {
//                             Some(inner_ty)
//                         } else {
//                             None
//                         }
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             };

//             if let Some(inner_type) = maybe_inner_type {
//                 quote! {
//                     pub fn #method_ident(&self) -> Option<&#inner_type> {
//                         match self {
//                             Self::#variant(ref opt) => opt.as_ref(),
//                             _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
//                         }
//                     }
//                 }
//             } else {
//                 quote! {
//                     pub fn #method_ident(&self) -> &#field_type {
//                         match self {
//                             Self::#variant(ref val) => val,
//                             _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
//                         }
//                     }
//                 }
//             }
//         }
//     }).collect();

//     let props_impl = quote! {
//         impl #new_struct_name {
//             /// Converts a Props variant to a key and its stringified value.
//             pub fn to_query_param(&self) -> (&'static str, BoltType) {
//                 match self {
//                     #(#to_query_param_match_arms),*
//                 }
//             }
    
//             // Accessor methods for the Props enum.
//             #(#props_accessor_methods)*
//         }
//     };

//     // Generate fields for the new struct: same field names, but type is the Props enum.
//     let new_struct_fields: Vec<_> = all_fields_full.iter().map(|field| {
//         let field_ident = field.ident.as_ref().unwrap();
//         let field_ty = &field.ty;
//         if should_ignore_field(field) {
//             quote! {
//                 pub #field_ident: #field_ty
//             }
//         } else {
//             quote! {
//                 pub #field_ident: #props_enum_name
//             }
//         }
//     }).collect();
    
//     // 2. Generate the constructor parameters (using the original types for all fields).
//     let constructor_params: Vec<_> = all_fields_full.iter().map(|field| {
//         let field_ident = field.ident.as_ref().unwrap();
//         let field_ty = &field.ty;
//         quote! {
//             #field_ident: #field_ty
//         }
//     }).collect();
    
//     // 3. Generate a list of field identifiers for forwarding.
//     let constructor_args: Vec<_> = all_fields_full.iter().map(|field| {
//         let field_ident = field.ident.as_ref().unwrap();
//         quote! { #field_ident }
//     }).collect();
    
//     // 4. Generate the constructor body. For non-ignored fields, we wrap the value in the
//     // corresponding Props enum variant; for ignored fields, we simply pass the value through.
//     let constructor_body: Vec<_> = all_fields_full.iter().map(|field| {
//         let field_ident = field.ident.as_ref().unwrap();
//         if should_ignore_field(field) {
//             quote! {
//                 #field_ident: #field_ident
//             }
//         } else {
//             let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
//             quote! {
//                 #field_ident: #props_enum_name::#variant(#field_ident)
//             }
//         }
//     }).collect();

//     // Generate accessor methods for the new struct.
//     // These delegate to the corresponding Props accessor methods.
// let struct_accessor_methods: Vec<_> = all_fields_full.iter().map(|field| {
//         if should_ignore_field(field) {
//             quote! {}
//         } else {
//             let field_ident = field.ident.as_ref().unwrap();
//             let field_type = &field.ty;
//             let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
//             // Check if field_type is Option<T>
//             let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
//                 if type_path.qself.is_none()
//                     && type_path.path.segments.len() == 1
//                     && type_path.path.segments[0].ident == "Option"
//                 {
//                     if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                         &type_path.path.segments[0].arguments
//                     {
//                         if let Some(syn::GenericArgument::Type(inner_ty)) =
//                             angle_bracketed.args.first()
//                         {
//                             Some(inner_ty)
//                         } else {
//                             None
//                         }
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             };
        
//             if let Some(inner_ty) = maybe_inner_type {
//                 // Field is optional: return Option<&InnerType>
//                 quote! {
//                     pub fn #method_ident(&self) -> Option<&#inner_ty> {
//                         self.#field_ident.#method_ident()
//                     }
//                 }
//             } else {
//                 // Field is required: return &FieldType
//                 quote! {
//                     pub fn #method_ident(&self) -> &#field_type {
//                         self.#field_ident.#method_ident()
//                     }
//                 }
//             }
//         }
//     }).collect();
    
//     // Generate a constructor method for the generated struct.
//     let generated_constructor = quote! {
//         impl #new_struct_name {
//             pub fn new( #(#constructor_params),* ) -> Self {
//                 Self {
//                     #(#constructor_body),*
//                 }
//             }
//         }
//     };

//     // Generate the new() method for the template struct that forwards to the generated struct's new().
//     let template_new_method = quote! {
//         impl #struct_name {
//             pub fn new( #(#constructor_params),* ) -> #new_struct_name {
//                 #new_struct_name::new( #(#constructor_args),* )
//             }
//         }
//     };

//     // Generate the impl block for the new struct with accessor methods
//     let struct_impl = quote! {
//         impl #new_struct_name {
//             #(#struct_accessor_methods)*
//         }
//     };

//         // Generate field initializers for the From<relation> impl.
//         let field_inits: Vec<_> = all_fields_full.iter().map(|(field)| {
//             let field_ident = field.ident.as_ref().unwrap();
//             let field_type = field.ty.clone();
        
//             if should_ignore_field(field) {
//                 // For ignored fields, we just use their default value.
//                 quote! {
//                     #field_ident: Default::default()
//                 }
//             } else {
//                 // For fields that go into queries, wrap them in the Props enum.
//                 let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
//                 let key = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
//                 let field_type_str = field_type.to_token_stream().to_string();
        
//                 // Generate extraction expression based on the type.
//                 let extraction = if field_type_str == "String" {
//                     quote! {
//                         relation.get(#key).unwrap_or_default()
//                     }
//                 } else if ["i8", "i16", "i32", "i64", "i128",
//                           "u8", "u16", "u32", "u64", "u128"]
//                     .contains(&field_type_str.as_str())
//                 {
//                     quote! {
//                         {
//                             let tmp: u64 = relation.get(#key).unwrap_or_default();
//                             tmp as #field_type
//                         }
//                     }
//                 } else if field_type_str == "bool" {
//                     quote! {
//                         relation.get(#key).unwrap_or_default()
//                     }
//                 } else if field_type_str == "f32" || field_type_str == "f64" {
//                     quote! {
//                         {
//                             let tmp: f64 = relation.get(#key).unwrap_or_default();
//                             tmp as #field_type
//                         }
//                     }
//                 } else {
//                     quote! {
//                         relation.get(#key).unwrap_or_default()
//                     }
//                 };
        
//                 quote! {
//                     #field_ident: #props_enum_name::#variant(#extraction)
//                 }
//             }
//         }).collect();

//         // Generate the complete From<Relation> implementation for the struct.
//         let from_impl = quote! {
//             impl From<Relation> for #new_struct_name {
//                 fn from(relation: Relation) -> Self {
//                     Self {
//                         #(#field_inits),*
//                     }
//                 }
//             }
//         };

//         let silly_from_impl = quote! {
//             impl From<relation> for #new_struct_name {
//                 fn from(relation: relation) -> Self {
//                     Self {
//                         #(#field_inits),*
//                     }
//                 }
//             }
//         };

//         // Generate query functions using the generated Props enum.
//         let get_relation_entity_type_fn = generators::generate_get_relation_entity_type();
//         //let get_relation_by_fn = generators::generate_get_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
//         let relation_by_fn = generators::generate_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
//         let get_relation_label_fn = generators::generate_get_relation_label(&new_struct_name_str);

//     // Assemble the final output.
//     let expanded = quote! {
//         // Generated Props enum.
//         #[derive(Debug, Clone)]
//         pub enum #props_enum_name {
//             #(#props_enum_variants),*
//         }

//         impl #props_enum_name {
//             /// Converts a Props variant to a key and its stringified value.
//             pub fn to_query_param(&self) -> (&'static str, BoltType) {
//                 match self {
//                     #(#to_query_param_match_arms),*
//                 }
//             }

//             // Accessor methods for the Props enum.
//             #(#props_accessor_methods)*
//         }

//         // Generated new struct (e.g., `User` from `UserTemplate`) whose fields are wrapped in the Props enum.
//         #[derive(Debug, Clone)]
//         pub struct #new_struct_name {
//             #(#new_struct_fields),*
//         }

//         // Implement the Neo4gEntity trait from neo4g_traits.
//         impl Neo4gEntity for #new_struct_name {
//             type Props = #props_enum_name;

//             fn get_entity_type(&self) -> String {
//                 Self::get_relation_entity_type()
//             }

//             fn get_label(&self) -> String {
//                 Self::get_relation_label()
//             }
            
//             // fn match_by(&self, props: &[Self::Props]) -> (String, String, std::collections::HashMap<String, BoltType>) {
//             //     Self::get_relation_by(props)
//             // }
            
//             fn entity_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
//                 Self::relation_by(props)
//             }

//             fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
//                 self.create_relation_from_self()
//             }
//         }
        
//         impl #new_struct_name {
//             #get_relation_entity_type_fn
//             //#get_relation_by_fn
//             #relation_by_fn
//             #create_relation_from_self_fn
//             #get_relation_label_fn
//         }

//         // Constructor for the generated struct.
//         #generated_constructor

//         // New() method for the template struct that forwards to the generated struct's new().
//         #template_new_method

//         #from_impl
//         #silly_from_impl

//         // Accessor methods for the generated struct.
//         #struct_impl
//     };

//     TokenStream::from(expanded)
// }











// use proc_macro::TokenStream;
// use quote::{quote, ToTokens};
// use syn::{parse_macro_input, DeriveInput, Data, Fields};
// //use neo4g_traits::*;
// use crate::{generators, utils};

// pub fn generate_neo4g_relation(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree.
//     let input = parse_macro_input!(input as DeriveInput);
//     let struct_name = &input.ident;
//     let struct_name_str = struct_name.to_string();

//     // Collect each field's identifier and type from the template struct.
//     let fields: Vec<(&syn::Ident, syn::Type)> = if let Data::Struct(data_struct) = &input.data {
//         if let Fields::Named(fields) = &data_struct.fields {
//             fields
//                 .named
//                 .iter()
//                 .map(|f| (f.ident.as_ref().unwrap(), f.ty.clone()))
//                 .collect()
//         } else {
//             vec![]
//         }
//     } else {
//         vec![]
//     };

//     // Generate the base name by removing the "Template" suffix (if present).
//     let base_name = struct_name_str.trim_end_matches("Template");
//     // Generated Props enum (e.g. UserProps).
//     let props_enum_name = syn::Ident::new(&format!("{}Props", base_name), struct_name.span());

//     // Generate enum variants that hold the actual field types.
//     let props_enum_variants: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
//         let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
//         quote! { #variant(#field_type) }
//     }).collect();

//     // Generate match arms for converting a variant’s inner value to a query parameter.
//     // let to_query_param_match_arms: Vec<_> = fields.iter().map(|(field_ident, _)| {
//     //     let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
//     //     let key = syn::LitStr::new(&field_ident.to_string(), struct_name.span());
//     //     quote! {
//     //         #props_enum_name::#variant(val) => (#key, val.to_string())
//     //     }
//     // }).collect();
//     // Generate match arms for converting a variant’s inner value to a query parameter.
//     let to_query_param_match_arms: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
//         let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
//         let key = syn::LitStr::new(&field_ident.to_string(), struct_name.span());

//         // Convert the field type into a string so we can match on it.
//         let field_type_str = field_type.to_token_stream().to_string();

//         // Determine the correct conversion for each type.
//         let conversion = if field_type_str == "String" {
//             quote! {
//                 BoltType::String(BoltString::from(val.clone()))
//             }
//         } else if field_type_str == "i32" {
//             quote! {
//                 BoltType::Integer(BoltInteger::from(*val))
//             }
//         } else {
//             // Fallback: convert the value to a string and wrap it in a BoltType::String.
//             quote! {
//                 BoltType::String(BoltString::from(val.to_string()))
//             }
//         };

//         quote! {
//             #props_enum_name::#variant(val) => (#key, #conversion)
//         }
//     }).collect();

//     // Generate accessor methods for the Props enum.
//     // For non-optional fields, return &T; for Option<T>, return Option<&T>.
//     let props_accessor_methods: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
//         let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
//         let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());

//         // Check if the field type is Option<T>.
//         let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
//             if type_path.qself.is_none()
//                 && type_path.path.segments.len() == 1
//                 && type_path.path.segments[0].ident == "Option"
//             {
//                 if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                     &type_path.path.segments[0].arguments
//                 {
//                     if let Some(syn::GenericArgument::Type(inner_ty)) =
//                         angle_bracketed.args.first()
//                     {
//                         Some(inner_ty)
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//         } else {
//             None
//         };

//         if let Some(inner_type) = maybe_inner_type {
//             quote! {
//                 pub fn #method_ident(&self) -> Option<&#inner_type> {
//                     match self {
//                         Self::#variant(ref opt) => opt.as_ref(),
//                         _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
//                     }
//                 }
//             }
//         } else {
//             quote! {
//                 pub fn #method_ident(&self) -> &#field_type {
//                     match self {
//                         Self::#variant(ref val) => val,
//                         _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
//                     }
//                 }
//             }
//         }
//     }).collect();

//     // Generate the new struct name by removing "Template" from the original struct name.
//     let new_struct_name = syn::Ident::new(base_name, struct_name.span());

//     // Generate fields for the new struct: same field names, but type is the Props enum.
//     let new_struct_fields: Vec<_> = fields.iter().map(|(field_ident, _)| {
//         quote! {
//             pub #field_ident: #props_enum_name
//         }
//     }).collect();

//     // Generate the constructor parameters (with the original types).
//     let constructor_params: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
//         quote! {
//             #field_ident: #field_type
//         }
//     }).collect();

//     // Generate a list of field identifiers for forwarding.
//     let constructor_args: Vec<_> = fields.iter().map(|(field_ident, _)| {
//         quote! { #field_ident }
//     }).collect();

//     // Generate the constructor body that wraps each field value in the corresponding Props variant.
//     let constructor_body: Vec<_> = fields.iter().map(|(field_ident, _)| {
//         let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
//         quote! {
//             #field_ident: #props_enum_name::#variant(#field_ident)
//         }
//     }).collect();

//     // Generate accessor methods for the new struct.
//     // These delegate to the corresponding Props accessor methods.
//     let struct_accessor_methods: Vec<_> = fields.iter().map(|(field_ident, field_type)| {
//         let method_ident = syn::Ident::new(&field_ident.to_string(), struct_name.span());
//         // Check if field_type is Option<T>
//         let maybe_inner_type = if let syn::Type::Path(type_path) = field_type {
//             if type_path.qself.is_none()
//                 && type_path.path.segments.len() == 1
//                 && type_path.path.segments[0].ident == "Option"
//             {
//                 if let syn::PathArguments::AngleBracketed(angle_bracketed) =
//                     &type_path.path.segments[0].arguments
//                 {
//                     if let Some(syn::GenericArgument::Type(inner_ty)) =
//                         angle_bracketed.args.first()
//                     {
//                         Some(inner_ty)
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//         } else {
//             None
//         };
    
//         if let Some(inner_ty) = maybe_inner_type {
//             // Field is optional: return Option<&InnerType>
//             quote! {
//                 pub fn #method_ident(&self) -> Option<&#inner_ty> {
//                     self.#field_ident.#method_ident()
//                 }
//             }
//         } else {
//             // Field is required: return &FieldType
//             quote! {
//                 pub fn #method_ident(&self) -> &#field_type {
//                     self.#field_ident.#method_ident()
//                 }
//             }
//         }
//     }).collect();
    
//     // Generate a constructor method for the generated struct.
//     let generated_constructor = quote! {
//         impl #new_struct_name {
//             pub fn new( #(#constructor_params),* ) -> Self {
//                 Self {
//                     #(#constructor_body),*
//                 }
//             }
//         }
//     };

//     // Generate the new() method for the template struct that forwards to the generated struct's new().
//     let template_new_method = quote! {
//         impl #struct_name {
//             pub fn new( #(#constructor_params),* ) -> #new_struct_name {
//                 #new_struct_name::new( #(#constructor_args),* )
//             }
//         }
//     };

//     // Generate the impl block for the new struct with accessor methods.
//     let new_struct_name_str = new_struct_name.to_string();
//     let struct_impl = quote! {
//         impl #new_struct_name {
//             #(#struct_accessor_methods)*
//         }
//     };

//     // Generate query functions using the generated Props enum.
//     let get_relation_entity_type_fn = generators::generate_get_relation_entity_type();
//     let get_relation_by_fn = generators::generate_get_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
//     let merge_relation_by_fn = generators::generate_merge_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);

//     // Assemble the final output.
//     let expanded = quote! {
//         // Generated Props enum.
//         #[derive(Debug, Clone)]
//         pub enum #props_enum_name {
//             #(#props_enum_variants),*
//         }

//         impl #props_enum_name {
//             /// Converts a Props variant to a key and its stringified value.
//             pub fn to_query_param(&self) -> (&'static str, BoltType) {
//                 match self {
//                     #(#to_query_param_match_arms),*
//                 }
//             }

//             // Accessor methods for the Props enum.
//             #(#props_accessor_methods)*
//         }

//         // Generated new struct (e.g., `User` from `UserTemplate`) whose fields are wrapped in the Props enum.
//         #[derive(Debug, Clone)]
//         pub struct #new_struct_name {
//             #(#new_struct_fields),*
//         }

//         // Implement the Neo4gEntity trait from neo4g_traits.
//         impl Neo4gEntity for #new_struct_name {
//             type Props = #props_enum_name;

//             fn get_entity_type(&self) -> String {
//                 Self::get_relation_entity_type()
//             }
            
//             fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
//                 Self::get_relation_by(props)
//             }
            
//             fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
//                 Self::merge_relation_by(props)
//             }
//         }
        
//         impl #new_struct_name {
//             #get_relation_entity_type_fn
//             #get_relation_by_fn
//             #merge_relation_by_fn
//         }

//         // Constructor for the generated struct.
//         #generated_constructor

//         // New() method for the template struct that forwards to the generated struct's new().
//         #template_new_method

//         // Accessor methods for the generated struct.
//         #struct_impl
//     };

//     TokenStream::from(expanded)
// }
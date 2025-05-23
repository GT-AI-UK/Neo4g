use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use crate::generators;
use heck::ToPascalCase;

pub fn generate_neo4g_relation(input: TokenStream) -> TokenStream {
    let conditional_attr = if cfg!(feature = "leptos") {
        quote! { #[cfg(feature = "ssr")] }
    } else {
        quote! {}
    };
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
    let props_enum_variants: Vec<_> = all_fields_full.iter()
    .filter_map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
            Some(quote! { #variant(#field_type) })
        } else {
            None
        }
    })
    .collect();

    let props_enum_current_variants: Vec<_> = all_fields_full.iter()
    .filter_map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let variant = syn::Ident::new(&format!("Current{}", field_ident.to_string().to_pascal_case()), struct_name.span());
            Some(quote! { #variant })
        } else {
            None
        }
    })
    .collect();

    let get_current_match_arms: Vec<_> = all_fields_full.iter().filter_map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), struct_name.span());
            let current_variant = syn::Ident::new(&format!("Current{}", field_ident.to_string().to_pascal_case()), struct_name.span());
            Some(quote! { 
                #props_enum_name::#variant(_) => prop.clone(),
                #props_enum_name::#current_variant => self.#field_ident.clone(),
            })
        } else {
            None
        }
    }).collect();

    let get_current_fn = quote! {
        fn get_current(&self, prop: &Self::Props) -> Self::Props {
            match prop {
                #(#get_current_match_arms)*
            }
        }
    };

    let self_to_props_vec_iter: Vec<_> = all_fields_full.iter().filter_map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string();
            Some(quote! {
                props_vec.push(self.#field_ident.clone());
            })
        } else {
            None
        }
    }).collect();

    let self_to_props_fn = quote! {
        pub fn self_to_props(&self) -> Vec<#props_enum_name> {
            let mut props_vec: Vec<#props_enum_name> = Vec::new();
            #(#self_to_props_vec_iter)*
            props_vec
        }
    };

    let create_relation_params: Vec<_> = all_fields_full.iter().filter_map(|field| {
        if !should_ignore_field(field) {
            let field_ident = field.ident.as_ref().unwrap();
            let field_name = field_ident.to_string();
            // Create a literal string for the field name.
            let field_name_lit = syn::LitStr::new(&field_name, field_ident.span());
            // We assume the accessor method has the same name as the field.
            let access_method_ident = syn::Ident::new(&field_name, field_ident.span());
            Some(quote! {
                (#field_name_lit.to_string(), BoltType::from(self.#access_method_ident().clone()))
            })
        } else {
            None
        }
    }).collect();
    
    let create_relation_from_self_fn = quote! {
        pub fn create_relation_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
            let self_props = self.self_to_props();
            let mapped_self_props: Vec<&#props_enum_name> = self_props.iter().map(|prop| prop).collect();
            let sliced_props: &[&#props_enum_name] = &mapped_self_props;
            Neo4gEntity::entity_by(self, &Aliasable::get_alias(self), &self_props)
        }
    };

    // Generate match arms for converting a variant’s inner value to a query parameter.
    let to_query_param_match_arms: Vec<_> = all_fields_full.iter().filter_map(|(field)| {
        let field_ident = field.ident.as_ref().unwrap();
        let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
        let key_lit = syn::LitStr::new(&field_ident.to_string(), field_ident.span());
        
        if should_ignore_field(field) {
            // For ignored fields, provide a match arm that essentially does nothing.
            None
        } else {
            // For normal fields, return the key and the value.
            Some(quote! {
                #props_enum_name::#variant(val) => (#key_lit, val.clone().into())
            })
        }
    }).collect();
    
    // Generate accessor methods for the Props enum.
    // For non-optional fields, return &T; for Option<T>, return Option<&T>.
    let props_accessor_methods: Vec<_> = all_fields_full.iter().filter_map(|field| {
        if should_ignore_field(field) {
            None
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
                Some(quote! {
                    pub fn #method_ident(&self) -> Option<&#inner_type> {
                        match self {
                            Self::#variant(ref opt) => opt.as_ref(),
                            _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
                        }
                    }
                })
            } else {
                Some(quote! {
                    pub fn #method_ident(&self) -> &#field_type {
                        match self {
                            Self::#variant(ref val) => val,
                            _ => panic!("Called {} accessor on wrong variant", stringify!(#method_ident)),
                        }
                    }
                })
            }
        }
    }).collect();

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
    
        // Check if the field type is a String.
        let is_string = if let syn::Type::Path(type_path) = field_ty {
            type_path.path.segments.last().map(|seg| seg.ident == "String").unwrap_or(false)
        } else {
            false
        };
    
        // Choose &str for constructor if the field type is String.
        let arg_type = if is_string {
            quote! { &str }
        } else {
            quote! { #field_ty }
        };
    
        quote! {
            #field_ident: #arg_type
        }
    }).collect();

    // Generate the constructor body. For non-ignored fields, we wrap the value in the
    // corresponding Props enum variant; for ignored fields, we simply pass the value through.
    let constructor_body: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        
        if should_ignore_field(field) {
            quote! {
                #field_ident: #field_ident
            }
        } else {
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
            
            // Check if the field type is a String.
            let is_string = if let syn::Type::Path(type_path) = field_ty {
                type_path.path.segments.last().map(|seg| seg.ident == "String").unwrap_or(false)
            } else {
                false
            };
            
            // Use `.to_string()` if it's a String, otherwise use the parameter as-is.
            let value = if is_string {
                quote! { #field_ident.to_string() }
            } else {
                quote! { #field_ident }
            };
            
            quote! {
                #field_ident: #props_enum_name::#variant(#value)
            }
        }
    }).collect();

    let default_body: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
    
        // Detect if the field is NaiveDateTime
        let is_naive_datetime = match ty {
            syn::Type::Path(type_path) => {
                type_path.path.segments.last().map(|seg| seg.ident == "NaiveDateTime").unwrap_or(false)
                    && type_path.path.segments.iter().any(|seg| seg.ident == "chrono")
            }
            _ => false,
        };
    
        if should_ignore_field(field) {
            if is_naive_datetime {
                quote! {
                    #field_ident: chrono::Utc::now().naive_local()
                }
            } else {
                quote! {
                    #field_ident: Default::default()
                }
            }
        } else {
            let variant = syn::Ident::new(&field_ident.to_string().to_pascal_case(), field_ident.span());
            if is_naive_datetime {
                quote! {
                    #field_ident: #props_enum_name::#variant(chrono::Utc::now().naive_local())
                }
            } else {
                quote! {
                    #field_ident: #props_enum_name::#variant(Default::default())
                }
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
        #conditional_attr
        impl #new_struct_name {
            pub fn new( #(#constructor_params),* ) -> Self {
                Self {
                    alias: String::new(),
                    uuid: Uuid::new_v4(),
                    entity_type: EntityType::Relation,
                    #(#constructor_body),*
                }
            }
        }
    };

    let generated_default = quote! {
        #conditional_attr
        impl Default for #new_struct_name {
            fn default() -> Self {
                Self {
                    alias: String::new(),
                    uuid: Uuid::new_v4(),
                    entity_type: EntityType::Relation,
                    #(#default_body),*
                }
            }
        }
    };

    let template_constructor_body: Vec<_> = all_fields_full.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        // Determine if the field's type is a String.
        let is_string = if let syn::Type::Path(type_path) = field_ty {
            type_path.path.segments.last().map(|seg| seg.ident == "String").unwrap_or(false)
        } else {
            false
        };
    
        if is_string {
            quote! { #field_ident: #field_ident.to_string() }
        } else {
            quote! { #field_ident: #field_ident }
        }
    }).collect();

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
        #conditional_attr
        impl #new_struct_name {
            #(#struct_accessor_methods)*
        }
    };

        // Generate field initializers for the From<relation> impl.
        let field_inits: Vec<_> = all_fields_full.iter().map(|field| {
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
                        entity.get(#key).unwrap_or_default()
                    }
                } else if ["i8", "i16", "i32", "i64", "i128",
                          "u8", "u16", "u32", "u64", "u128"]
                    .contains(&field_type_str.as_str())
                {
                    quote! {
                        {
                            let tmp: u64 = entity.get(#key).unwrap_or_default();
                            tmp as #field_type
                        }
                    }
                } else if field_type_str == "bool" {
                    quote! {
                        entity.get(#key).unwrap_or_default()
                    }
                } else if field_type_str == "f32" || field_type_str == "f64" {
                    quote! {
                        {
                            let tmp: f64 = entity.get(#key).unwrap_or_default();
                            tmp as #field_type
                        }
                    }
                } else {
                    quote! {
                        entity.get(#key).unwrap_or_default()
                    }
                };
        
                quote! {
                    #field_ident: #props_enum_name::#variant(#extraction)
                }
            }
        }).collect();

        let from_db_entity_fn = quote! {
            pub fn from_db_entity(db_entity: DbEntityWrapper) -> EntityWrapper {
                if let DbEntityWrapper::Relation(entity) = db_entity {
                    EntityWrapper::#new_struct_name(
                        #new_struct_name {
                            alias: String::new(),
                            uuid: Uuid::new_v4(),
                            entity_type: EntityType::Relation,
                            #(#field_inits),*
                        }
                    )
                } else {
                    EntityWrapper::Nothing(Nothing::default())
                }
            }
        };

        let wrap_fn = quote! {
            pub fn wrap(&self) -> EntityWrapper {
                let obj = #new_struct_name {
                    ..self.clone()
                };
                EntityWrapper::#new_struct_name(obj)
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
            #conditional_attr
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
            #conditional_attr
            impl From<#struct_name> for #new_struct_name {
                fn from(template: #struct_name) -> Self {
                    Self {
                        alias: String::new(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Relation,
                        #(#from_template_fields),*
                    }
                }
            }
        };

        // Generate query functions using the generated Props enum.
        let get_relation_entity_type_fn = generators::generate_get_relation_entity_type();
        let relation_by_fn = generators::generate_relation_by(&new_struct_name, &new_struct_name_str, &props_enum_name);
        let get_relation_label_fn = generators::generate_get_relation_label(&new_struct_name_str);
        let set_alias_fn = generators::generate_set_alias();
        let get_alias_fn = generators::generate_get_alias();

    // Assemble the final output.
    let expanded = quote! {
        // Generated Props enum.
        #conditional_attr
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum #props_enum_name {
            #(#props_enum_variants),*,
            #(#props_enum_current_variants),*
        }

        #conditional_attr
        impl QueryParam for #props_enum_name {
            fn to_query_param(&self) -> (&'static str, BoltType) {
                match self {
                    #(#to_query_param_match_arms),*,
                    _ => ("nope", 0.into()),
                }
            }
        }

        #conditional_attr
        impl #props_enum_name {
            #(#props_accessor_methods)*
        }

        // Generated new struct (e.g., `User` from `UserTemplate`) whose fields are wrapped in the Props enum.
        #conditional_attr
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct #new_struct_name {
            pub alias: String,
            pub uuid: Uuid,
            pub entity_type: EntityType,
            #(#new_struct_fields),*
        }

        // Implement the Neo4gEntity trait from neo4g_traits.
        #conditional_attr
        impl Neo4gEntity for #new_struct_name {
            type Props = #props_enum_name;

            fn get_entity_type(&self) -> EntityType {
                self.entity_type.clone()
            }

            fn get_label(&self) -> String {
                Self::get_relation_label()
            }
            
            #get_current_fn
            
            fn entity_by(&self, alias: &str, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>) {
                Self::relation_by(alias, props)
            }

            fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>) {
                self.create_relation_from_self()
            }
        }

        #conditional_attr
        impl Aliasable for #new_struct_name {
            fn set_alias(&mut self, alias: &str) {
                self.set_entity_alias(alias);
            }
            fn get_alias(&self) -> String {
                self.get_entity_alias()
            }
            fn get_uuid(&self) -> Uuid {
                self.uuid.clone()
            }
        }

        #conditional_attr
        impl Paramable for #new_struct_name {
            fn to_query_uuid_param(&self) -> (String, Vec<Uuid>, HashMap<String, BoltType>) {
                (self.get_entity_alias(), Vec::new(), HashMap::new())
            }
        }
        
        #conditional_attr
        impl #new_struct_name {
            #get_relation_entity_type_fn
            #wrap_fn
            #relation_by_fn
            #create_relation_from_self_fn
            #get_relation_label_fn
            #set_alias_fn
            #get_alias_fn
            #self_to_props_fn
            #from_db_entity_fn
        }

        // Constructor for the generated struct.
        #generated_constructor
        #generated_default
        #template_new_method
        #to_template_impl
        #from_template_impl
        #struct_impl
    };

    TokenStream::from(expanded)
}


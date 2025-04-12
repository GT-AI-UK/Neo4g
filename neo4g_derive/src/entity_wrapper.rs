use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

pub fn generate_entity_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "Neo4gEntityWrapper can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let mut accessors = Vec::new();
    let mut from_node_checks = Vec::new();
    let mut from_relation_checks = Vec::new();
    let mut eq_checks = Vec::new();
    let mut call_get_alias_arms = Vec::new();
    let mut call_set_alias_arms = Vec::new();
    let mut call_get_entity_type_arms = Vec::new();
    let mut db_from_node_checks = Vec::new();
    let mut db_from_relation_checks = Vec::new();
    let mut call_get_uuid_arms = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let unwrap_fn_name = format_ident!("get_{}", var_name.to_string().to_lowercase());

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

        // Skip the Nothing variant for label checks.
        if var_name.to_string() == "Nothing" {
            continue;
        }
        let var_name_str = var_name.to_string();
        let check = quote! {
            if labels.contains(&#var_name_str) {
                return #enum_name::#var_name(#var_name::from(node));
            }
        };
        from_node_checks.push(check);
        let rcheck = quote! {
            if &labels.to_string().to_pascal_case() == &#var_name_str {
                return #enum_name::#var_name(#var_name::from(relation));
            }
        };
        from_relation_checks.push(rcheck);
        let eq_check = quote! {
            (#enum_name::#var_name(_), #enum_name::#var_name(_)) => true,
        };
        eq_checks.push(eq_check);
        let call_get_alias_arm = quote! {
            #enum_name::#var_name(inner) => inner.get_alias(),
        };
        call_get_alias_arms.push(call_get_alias_arm);
        let call_set_alias_arm = quote! {
            #enum_name::#var_name(inner) => inner.set_alias(alias),
        };
        call_set_alias_arms.push(call_set_alias_arm);
        let call_get_entity_type_arm = quote! {
            #enum_name::#var_name(inner) => inner.get_entity_type(),
        };
        call_get_entity_type_arms.push(call_get_entity_type_arm);
        let call_get_uuid_arm = quote! {
            #enum_name::#var_name(inner) => inner.get_uuid(),
        };
        call_get_uuid_arms.push(call_get_uuid_arm);

        let dbcheck = quote! {
            if labels.contains(&#var_name_str) {
                return #var_name::from_db_entity(db_entity);
            }
        };
        db_from_node_checks.push(dbcheck);
        let dbrcheck = quote! {
            if &labels.to_string().to_pascal_case() == &#var_name_str {
                return #var_name::from_db_entity(db_entity);
            }
        };
        db_from_relation_checks.push(dbrcheck);
    }

    let get_alias_fn = quote! {
        fn get_alias(&self) -> String {
            match self {
                #(#call_get_alias_arms)*
                _ => String::new()
            }
        }
    };
    let set_alias_fn = quote! {
        fn set_alias(&mut self, alias: &str) {
            match self {
                #(#call_set_alias_arms)*
                _ => ()
            }
        }
    };
    let get_entity_type_fn = quote! {
        fn get_entity_type(&self) -> EntityType {
            match self {
                #(#call_get_entity_type_arms)*
                _ => EntityType::Node
            }
        }
    };
    let get_entity_uuid_fn = quote! {
        fn get_uuid(&self) -> Uuid {
            match self {
                #(#call_get_uuid_arms)*
                _ => Uuid::new_v4()
            }
        }
    };

    let from_db_entity_fn = quote! {
        fn from_db_entity(db_entity: DbEntityWrapper) -> Self {
            match db_entity.clone() {
                DbEntityWrapper::Node(entity) => {
                    let labels = entity.labels();
                    #(#db_from_node_checks)*
                    return #enum_name::Nothing(Nothing::new(true));
                },
                DbEntityWrapper::Relation(entity) => {
                    let labels = entity.typ();
                    #(#db_from_relation_checks)*
                    return #enum_name::Nothing(Nothing::new(true));
               },
            }
        }
    };

    let gen = quote! {
        #(#accessors)*

        impl Aliasable for EntityWrapper {
            #get_alias_fn
            #set_alias_fn
            #get_entity_uuid_fn
        }

        impl WrappedNeo4gEntity for EntityWrapper {
            #from_db_entity_fn
            #get_entity_type_fn
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
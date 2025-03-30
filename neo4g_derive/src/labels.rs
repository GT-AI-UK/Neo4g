use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

pub fn generate_labels(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // Ensure that input.data is an enum, then get the variants.
    let data_enum = match input.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "Neo4gLabels can only be derived for enums"
            )
            .to_compile_error()
            .into();
        }
    };

    let mut from_node_checks = Vec::new();
    let mut from_relation_checks = Vec::new();
    let mut fmt_arms = Vec::new();

    for variant in data_enum.variants.iter() {
        let var_name = &variant.ident;
        let var_name_str = var_name.to_string();
        let check = quote! {
            if labels.contains(&#var_name_str) {
                return #enum_name::#var_name;
            }
        };
        from_node_checks.push(check);
        let rcheck = quote! {
            if &labels.to_string().to_pascal_case() == &#var_name_str {
                return #enum_name::#var_name;
            }
        };
        from_relation_checks.push(rcheck);
        
        let fmt_arm = quote! {
            #enum_name::#var_name => #var_name_str,
        };
        fmt_arms.push(fmt_arm);
    }

    // Generate the from_node function.
    let from_node_fn = quote! {
        pub fn from_node(node: Node) -> Self {
            let labels = node.labels();
            #(#from_node_checks)*
            // Fallback: if no label matched, return the Nothing variant.
            #enum_name::Nothing
        }
    };

    let from_relation_fn = quote! {
        pub fn from_relation(relation: Relation) -> Self {
            let labels = relation.typ();
            #(#from_relation_checks)*
            // Fallback: if no label matched, return the Nothing variant.
            #enum_name::Nothing
        }
    };

    let fmt_fn = quote! {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Formatter::write_str(f,
                match self {
                    #(#fmt_arms)*
                    _ => "",
                }
            )
        }
    };

    let gen = quote! {
        impl #enum_name {
            #from_node_fn
            #from_relation_fn
        }
        impl std::fmt::Display for #enum_name {
            #fmt_fn
        }
        impl Neo4gLabel for #enum_name {}
    };

    gen.into()
}
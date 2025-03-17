use proc_macro::TokenStream;
mod node;
mod relation;
mod generators;
mod utils;
mod entity_wrapper;
mod props_wrapper;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};
use syn::parse::Parse;

#[proc_macro_derive(Neo4gNode, attributes(not_query_param, skip_serde))]
pub fn neo4g_node_derive(input: TokenStream) -> TokenStream {
    node::generate_neo4g_node(input)
}

#[proc_macro_derive(Neo4gRelation)]
pub fn neo4g_relationship_derive(input: TokenStream) -> TokenStream {
    relation::generate_neo4g_relation(input)
}

#[proc_macro_attribute]
pub fn not_query_param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the input unmodified.
    item
}


#[proc_macro_derive(Neo4gEntityWrapper)] // New macro
pub fn neo4g_entity_derive(input: TokenStream) -> TokenStream {
    entity_wrapper::generate_entity_wrapper(input)
}

#[proc_macro_derive(Neo4gPropsWrapper)] // New macro
pub fn neo4g_props_derive(input: TokenStream) -> TokenStream {
    props_wrapper::generate_props_wrapper(input)
}
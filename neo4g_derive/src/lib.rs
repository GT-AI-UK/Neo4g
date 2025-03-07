use proc_macro::TokenStream;
mod node;
mod relation;
mod generators;
mod utils;
mod entity_wrapper;

#[proc_macro_derive(Neo4gNode)]
pub fn neo4g_node_derive(input: TokenStream) -> TokenStream {
    node::generate_neo4g_node(input)
}

// #[proc_macro_derive(Neo4gRelationship)]
// pub fn neo4g_relationship_derive(input: TokenStream) -> TokenStream {
//     relationship::generate_neo4g_relationship(input)
// }

#[proc_macro_derive(Neo4gEntityWrapper)] // New macro
pub fn neo4g_entity_derive(input: TokenStream) -> TokenStream {
    entity_wrapper::generate_entity_wrapper(input)
}

// #[proc_macro_derive(Neo4gPropsWrapper)] // New macro
// pub fn neo4g_props_derive(input: TokenStream) -> TokenStream {
//     props_wrapper::generate_props_wrapper(input)
// }
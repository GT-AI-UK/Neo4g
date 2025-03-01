use proc_macro::TokenStream;
mod node;
mod relationship;
mod generators;
mod utils;

#[proc_macro_derive(Neo4gNode)]
pub fn neo4g_node_derive(input: TokenStream) -> TokenStream {
    node::generate_neo4g_node(input)
}

#[proc_macro_derive(Neo4gRelationship)]
pub fn neo4g_relationship_derive(input: TokenStream) -> TokenStream {
    relationship::generate_neo4g_relationship(input)
}
use proc_macro::TokenStream;
mod node;
mod relation;
mod generators;
mod utils;
mod entity_wrapper;
mod props_wrapper;
mod labels;
mod prop;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};
use syn::parse::Parse;

/// An ORM-ish approach to Neo4j. Neo4gNode is to be used on a struct whose name ends in "Template".
/// This is because it creates the following:
/// - A new struct with "Template" removed from the original name
/// - An enum that wraps all the struct properties
/// - Various trait impls, including conversions between the Template and derived version of the struct
/// All of this is to create an object that is compatible with the Neo4gBuider struct which provides convenient and rusty access to neo4j.
#[proc_macro_derive(Neo4gNode, attributes(not_query_param, skip_serde))]
pub fn neo4g_node_derive(input: TokenStream) -> TokenStream {
    node::generate_neo4g_node(input)
}

/// An ORM-ish approach to Neo4j. Neo4gRelation is to be used on a struct whose name ends in "Template".
/// This is because it creates the following:
/// - A new struct with "Template" removed from the original name
/// - An enum that wraps all the struct properties
/// - Various trait impls, including conversions between the Template and derived version of the struct
/// All of this is to create an object that is compatible with the Neo4gBuider struct which provides convenient and rusty access to neo4j.
#[proc_macro_derive(Neo4gRelation)]
pub fn neo4g_relationship_derive(input: TokenStream) -> TokenStream {
    relation::generate_neo4g_relation(input)
}

/// This marker is used to prevent a struct attribute from being added to the Props enum.
/// This is useful for Vec objects and other things that don't easily convert to BoltTypes.
#[proc_macro_attribute]
pub fn not_query_param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Just a marker - nothing changes
    item
}

/// This is used to provide necesary impls for an enum that is used in a struct's property.
/// It provides formatting, a from(String), a BoltType::from(enum), and a trait that ensures dependencies are met.
#[proc_macro_derive(Neo4gProp)]
pub fn neo4g_prop_derive(input: TokenStream) -> TokenStream {
    prop::generate_neo4g_prop(input)
}

/// This is used by the macro rules macro that generates enums to generate the impl blocks for the enums.
#[proc_macro_derive(Neo4gEntityWrapper)]
pub fn neo4g_entity_derive(input: TokenStream) -> TokenStream {
    entity_wrapper::generate_entity_wrapper(input)
}

/// This is used by the macro rules macro that generates enums to generate the impl blocks for the enums.
#[proc_macro_derive(Neo4gPropsWrapper)]
pub fn neo4g_props_derive(input: TokenStream) -> TokenStream {
    props_wrapper::generate_props_wrapper(input)
}

/// This is used by the macro rules macro that generates enums to generate the impl blocks for the enums.
#[proc_macro_derive(Neo4gLabels)]
pub fn neo4g_labels_derive(input: TokenStream) -> TokenStream {
    labels::generate_labels(input)
}
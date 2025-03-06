use neo4g_macro_rules::{generate_props_wrapper, generate_entity_wrapper};
use paste::paste;
use crate::objects::{User, Group};
use neo4g_derive::Neo4gEntityWrapper;

generate_entity_wrapper!(User, Group);
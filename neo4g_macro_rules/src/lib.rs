use paste::paste;
use neo4g_derive::Neo4gEntityWrapper;

#[macro_export]
macro_rules! generate_entity_wrapper {
    ( $( $struct_name:ident ),* $(,)? ) => {
        paste! {
            #[derive(Debug, Clone, Neo4gEntityWrapper)]
            pub enum EntityWrapper {
                $(
                    $struct_name($struct_name),
                )*
            }
        }
    }
}


#[macro_export]
macro_rules! generate_props_wrapper {
    ( $( $struct_name:ident ),* ) => {
        paste! {
            #[derive(Debug, Clone)]
            pub enum PropsWrapper {
                $(
                    [<$struct_name Props>]([<$struct_name Props>]),
                )*
                None,
                Error(String),
            }
        }
    }
}
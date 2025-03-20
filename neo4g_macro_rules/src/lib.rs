use paste::paste;
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper};

#[macro_export]
macro_rules! generate_entity_wrappers {
    ( $( $struct_name:ident ),* $(,)? ) => {
        paste! {
            #[derive(Debug, Clone, Neo4gEntityWrapper)]
            pub enum EntityWrapper {
                $(
                    $struct_name($struct_name),
                )*
            }
        }
        paste! {
            #[derive(Debug, Clone, Neo4gPropsWrapper)]
            pub enum PropsWrapper {
                $(
                    [<$struct_name Props>]([<$struct_name Props>]),
                )*
            }
        }
        paste! {
            #[derive(Debug, Clone)]
            pub enum Label {
                $(
                    $struct_name,
                )*
            }
        }
    }
}
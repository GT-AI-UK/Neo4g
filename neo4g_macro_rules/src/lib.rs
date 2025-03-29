use paste::paste;
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper, Neo4gLabels};

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
            #[derive(Debug, Clone, Neo4gLabels)]
            pub enum Label {
                Any,
                SysObj,
                $(
                    $struct_name,
                )*
            }
        }
    }
}

// could do macro rules for returns!, take in the var names, and output them wrapped in EntityWrapper in a slice, with the EntityWrapper::from_db_entity() function after?
// doesn't solve the issue, but makes everything more convenient...
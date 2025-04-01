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

#[macro_export]
macro_rules! wrap {
    ($($arg:expr),* $(,)?) => {
        &[
            $( &$arg.wrap() ),*
        ]
    }
}

#[macro_export]
macro_rules! props {
    ($entity:ident => $($field:expr),* $(,)?) => {
        |$entity| vec![$($field.clone()),*]
    };
}

#[macro_export]
macro_rules! prop {
    ($entity:ident . $field:ident) => {
        |$entity| $entity.$field.clone()
    };
}

#[macro_export]
macro_rules! no_props {
    () => {
        |_| Vec::new()
    };
}
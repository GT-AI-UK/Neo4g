use paste::paste;
use neo4g_derive::{Neo4gEntityWrapper, Neo4gPropsWrapper, Neo4gLabels};

/// Required for Neo4gNode and Neo4gRelation to work.
/// Generates EntityWrapper, PropsWrapper, and Label enums.
/// Also generates impl blocks for each using derive macros.
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

/// Calls .wrap() on provided args. Useful to create an &[EntityWrapper].
#[macro_export]
macro_rules! wrap {
    [$($arg:expr),* $(,)?] => {
        &[
            $( $arg.wrap() ),*
        ]
    }
}

/// Generates a closure to access entity props conveniently. Use with .node(), .relation(), etc.
/// # Example:
/// ```rust
/// props!(entity => entity.prop, EntityProps::Prop(val))
/// ```
/// The example above generates:
/// ```rust
/// |entity| vec![entity.prop.clone(), entity.prop.clone()]
/// ``` 
#[macro_export]
macro_rules! props {
    ($entity:ident => $($field:expr),* $(,)?) => {
        |$entity| vec![$($field.clone()),*]
    };
}

/// Generates a closure to access entity props conveniently. Use with .condition().
/// # Example:
/// ```rust
/// prop!(entity.prop)
/// ```
/// The example above generates:
/// ```rust
/// |entity| entity.field1.clone()
/// ```
/// NOTE: There is no alternative for EntityProps::Prop(val) because it's simple enough to just write:
/// ```rust
/// |_| EntityProps::Prop(val)
/// ```
#[macro_export]
macro_rules! prop {
    ($entity:ident . $field:ident) => {
        |$entity| $entity.$field.clone()
    };
}

/// Generates an empty closure that returns an empty vec. Use with .node(), .relation(), etc. when no props are required.
#[macro_export]
macro_rules! no_props {
    () => {
        |_| Vec::new()
    };
}

/// Generates input for the arrays argument in .with_arrays().
/// # Exmaple:
/// ```rust
/// arrays![array1, array2, array_n]
/// ```
/// The exmaple above generates:
/// ```rust
/// &mut [&mut array1, &mut array2, &mut array_n]
/// ```
#[macro_export]
macro_rules! arrays {
    [$($arg:expr),* $(,)?] => {
        &mut [
            $( &mut $arg ),*
        ]
    };
}

// probably need a macro rules macro for numbers![] to allow easy .into() calls to BoltType for all numeric types.
// this would allow functions to be used more easily? might have to format and put into the query manually as a bolt vec is still a rust vec?
// hmmmmm
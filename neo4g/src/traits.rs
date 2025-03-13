use std::collections::HashMap;

use neo4rs::BoltType;

pub trait Neo4gEntity {
    type Props;
    fn get_entity_type(&self) -> String;
    fn get_label(&self) -> String;
    //fn match_by(&self, props: &[Self::Props]) -> (String, String, std::collections::HashMap<String, BoltType>);
    fn entity_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, BoltType>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
}

pub trait Neo4gProp: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn key(&self) -> &'static str;
    fn value(&self) -> String;
}

pub trait Neo4gEntityObjectSafe {
    fn get_entity_type(&self) -> String;
    fn get_label(&self) -> String;
    // fn match_by_obj(&self, props: &[Box<dyn Neo4gProp>])
    //     -> (String, String, std::collections::HashMap<String, BoltType>);
    fn entity_by_obj(&self, props: &[Box<dyn Neo4gProp>])
        -> (String, std::collections::HashMap<String, BoltType>);
    fn create_from_self(&self) -> (String, std::collections::HashMap<String, BoltType>);
}
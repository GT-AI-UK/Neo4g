//need to be moved into another crate so that both neo4g and neo4g_macros can depend on it!

pub trait Neo4gEntity {
    type Props;
    fn match_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
    fn merge_by(&self, props: &[Self::Props]) -> (String, std::collections::HashMap<String, String>);
}

pub trait Neo4gProp: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn key(&self) -> &'static str;
    fn value(&self) -> String;
}

pub trait ObjectSafeNeo4gEntity {
    fn match_by_obj(&self, props: &[Box<dyn Neo4gProp>])
        -> (String, std::collections::HashMap<String, String>);
    fn merge_by_obj(&self, props: &[Box<dyn Neo4gProp>])
        -> (String, std::collections::HashMap<String, String>);
}
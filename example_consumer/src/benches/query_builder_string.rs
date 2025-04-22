use crate::entity_wrapper::{EntityWrapper, Label};
use crate::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, CompOper, Neo4gBuilder, Where};
use neo4g_macro_rules::{no_props, prop, props};
use chrono::{NaiveDateTime, Utc};
use neo4rs::Graph;
use dotenv::dotenv;
use std::{env, result};
use heck::ToShoutySnakeCase;

pub fn query_builder_string_bench() {
    let mut component1 = Component::new("cid3", "path3", ComponentType::Type1, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut component2 = Component::new("cid73", "path16", ComponentType::Type2, Utc::now().naive_local(), Utc::now().naive_local(), false);
    let mut hcrel1 = HasComponent::default();
    let mut hcrel2 = HasComponent::default();
    let mut page1 = Page::new("pid4", "ppath4", vec![component1.clone().into(), component2.clone().into()], Utc::now().naive_local(), Utc::now().naive_local(), false);
    let result = Neo4gBuilder::new()
        .get()
            .node(&mut page1, props!(page1 => page1.id))
            .relation(&mut hcrel1, no_props!())
            .node(&mut component1, props!(component1 => component1.id))
            .filter(Where::new()
                .condition_prop(&page1, Some(&page1.id), CompareOperator::by_prop(CompOper::Eq, &page1.id, query_builder::RefType::Val))
            )
        .end_statement()
        .build();
}
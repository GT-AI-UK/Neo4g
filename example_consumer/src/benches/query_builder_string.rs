use crate::entity_wrapper::{EntityWrapper, Label};
use crate::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, Neo4gBuilder, Where};
use neo4g_macro_rules::{no_props, prop, props};
use neo4rs::Graph;
use dotenv::dotenv;
use std::{env, result};
use heck::ToShoutySnakeCase;

pub fn query_builder_string_bench() {
    let mut component1 = Component::new("cid3", "path3", ComponentType::Type1);
    let mut component2 = Component::new("cid73", "path16", ComponentType::Type2);
    let mut hcrel1 = HasComponent::default();
    let mut hcrel2 = HasComponent::default();
    let mut page1 = Page::new("pid4", "ppath4", vec![component1.clone(), component2.clone()]);
    let result = Neo4gBuilder::new()
        .get()
            .node(&mut page1, props!(page1 => page1.id))
            .relation(&mut hcrel1, no_props!())
            .node(&mut component1, props!(component1 => component1.id))
            .filter(Where::new()
                .condition(&page1, &page1.id, CompareOperator::Eq)
                .join(CompareJoiner::And)
                .nest(|inner| {
                    inner.condition(&component1, &ComponentProps::Id("asdfasdf".into()), CompareOperator::Ne)
                    .join(CompareJoiner::And)
                    .condition(&component2, &ComponentProps::Id("asdfasdfasdf".into()), CompareOperator::Ne)
                })
            )
        .end_statement()
        .build();
}
use crate::entity_wrapper::{EntityWrapper, Label};
use crate::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, Neo4gBuilder, Where};
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
            .node(&mut page1, &[&PageProps::Id("pid99".to_string())])
            .relation(&mut hcrel1, &[])
            .node(&mut component1, &[&ComponentProps::Id("cid3".to_string())])
        .end_statement()
        .get()
            .node_ref(&page1)
            .relation(&mut hcrel2, &[])
            .node(&mut component2, &[&ComponentProps::Id("cid73".to_string())])
            .filter(Where::new()
                .condition(&page1, &PageProps::Id("pid99".into()).into(), CompareOperator::Eq)
                .join(CompareJoiner::And)
                .nest(|parent_filter| parent_filter, Where::new()
                    .condition(&component1, &ComponentProps::Id("pid99".into()).into(), CompareOperator::Ne)
                    .join(CompareJoiner::And)
                    .condition(&component2, &ComponentProps::Id("pid99".into()).into(), CompareOperator::Ne)
                )          
            )
            .end_statement()
    .build();
}
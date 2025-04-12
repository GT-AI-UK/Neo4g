use crate::entity_wrapper::{EntityWrapper, Label};
use crate::objects::{Group, GroupProps, MemberOf, MemberOfProps, User, UserProps, UserTemplate, Page, PageProps, PageTemplate, Component, ComponentProps, ComponentTemplate, ComponentType, HasComponent, HasComponentTemplate, HasComponentProps};
use neo4g::query_builder::{self, CompareJoiner, CompareOperator, EntityType, Neo4gBuilder, Where};
use neo4rs::{query, Graph, BoltString, Node, Relation};
use dotenv::dotenv;
use uuid::Uuid;
use std::{env, result};
use heck::ToShoutySnakeCase;

pub async fn connect_neo4j() -> Graph { //return db object, run on startup, bind to state var
    let mut host = String::new();
    let mut port = String::new();
    let mut db_user = String::new();
    let mut db_password = String::new();
    dotenv().ok();
    if let Ok(env_host) = env::var("DB_HOST") {host = env_host.to_string();} else {println!("DB_HOST is not set in the .env file");}
    if let Ok(env_port) = env::var("DB_PORT") {port = env_port.to_string();} else {println!("DB_PORT is not set in the .env file");}
    if let Ok(env_db_user) = env::var("DB_USERNAME") {db_user = env_db_user.to_string();} else {println!("DB_USERNAME is not set in the .env file");}
    if let Ok(env_db_password) = env::var("DB_PASSWORD") {db_password = env_db_password.to_string();} else {println!("DB_PASSWORD is not set in the .env file");}
    let uri = format!("bolt://{}:{}", host, port);
    let graph = Graph::new(&uri, &db_user, &db_password).await.unwrap();
    graph
}

pub async fn static_query_bench() {
    let graph = connect_neo4j().await;
    let result = graph.execute(query("
        MATCH (page1:Page {id: $page1_id})-[has_component1:HAS_COMPONENT]->(component2:Component {id: $component2_id})
        MATCH (page1)-[has_component2:HAS_COMPONENT]->(component3:Component {id: $component3_id})
        WHERE page1.id = $where_id1 AND (component2.id <> $where_id1 AND component3.id <> $where_id2)
        RETURN page1, has_component1, component2, has_component2, component3
    ")
    .param("where_id2", "pid99")
    .param("page1_id", "pid99")
    .param("where_id1", "pid99")
    .param("component3_id", "cid73")
    .param("component2_id", "cid3")
    ).await;
    let mut entities: Vec<EntityWrapper> = Vec::new();
    if let Ok(mut r) = result {
        while let Ok(Some(row)) = r.next().await {
            if let Ok(node) = row.get::<Node>("page1") {
                if let (Ok(id), Ok(path)) = (node.get::<String>("id"), node.get::<String>("path")) {
                    let wrapped_entity = EntityWrapper::Page(Page {
                        alias: "page1".into(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Node,
                        id: PageProps::Id(id),
                        path: PageProps::Path(path),
                        components: Vec::new(),
                    });
                    entities.push(wrapped_entity);
                }
            }
            if let Ok(node) = row.get::<Relation>("has_component1") {
                if let Ok(deleted) = node.get::<bool>("deleted") {
                    let wrapped_entity = EntityWrapper::HasComponent(HasComponent {
                        alias: "has_component1".into(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Relation,
                        deleted: HasComponentProps::Deleted(deleted),
                    });
                    entities.push(wrapped_entity);
                }
            }
            if let Ok(node) = row.get::<Node>("component2") {
                if let (Ok(id), Ok(path), Ok(component_type)) = (node.get::<String>("id"), node.get::<String>("path"), node.get::<String>("component_type")) {
                    let wrapped_entity = EntityWrapper::Component(Component {
                        alias: "component2".into(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Node,
                        id: ComponentProps::Id(id),
                        path: ComponentProps::Path(path),
                        component_type: ComponentProps::ComponentType(component_type.into()),
                    });
                    entities.push(wrapped_entity);
                }
            }
            if let Ok(node) = row.get::<Relation>("has_component2") {
                if let Ok(deleted) = node.get::<bool>("deleted") {
                    let wrapped_entity = EntityWrapper::HasComponent(HasComponent {
                        alias: "has_component2".into(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Relation,
                        deleted: HasComponentProps::Deleted(deleted),
                    });
                    entities.push(wrapped_entity);
                }
            }
            if let Ok(node) = row.get::<Node>("component3") {
                if let (Ok(id), Ok(path), Ok(component_type)) = (node.get::<String>("id"), node.get::<String>("path"), node.get::<String>("component_type")) {
                    let wrapped_entity = EntityWrapper::Component(Component {
                        alias: "component3".into(),
                        uuid: Uuid::new_v4(),
                        entity_type: EntityType::Node,
                        id: ComponentProps::Id(id),
                        path: ComponentProps::Path(path),
                        component_type: ComponentProps::ComponentType(component_type.into()),
                    });
                    entities.push(wrapped_entity);
                }
            }
        }
    }
}
use neo4g::{Neo4gEntity, Neo4gEntityTrait};
use neo4rs::Node;

pub struct User {
    id: UserProps,
    name: UserProps,
}

impl User {
    fn new(id: i32, name: &str) -> Self {
        Self {
            id: UserProps::Id(id),
            name: UserProps::Name(name.to_string()),
        }
    }

    // fn from_entity(entity: Neo4gEntity) -> Self {

    // }

    fn get_id(&self) -> Option<i32> {
        if let UserProps::Id(id) = self.id {
            return Some(id);
        } else {
            return None;
        }
    }
    
    fn set_id(&mut self, id: i32) -> Self {
        self.id = UserProps::Id(id);
        self
    }

    fn get_name(&self) -> Option<String> {
        if let UserProps::Name(name) = self.name {
            return Some(name);
        } else {
            return None;
        }
    }

    fn set_name(&mut self, name: String) -> Self {
        self.name = UserProps::Name(name);
        self
    }
}

impl Neo4gEntityTrait for User {
    type Props = UserProps;

    fn get_name () -> String {
        "User"
    }
    fn create_node_from(user: User) -> (String, HashMap<String, BoltType>) {
        // destructure name from User
        let query = fomat!("CREATE (neo4g_node:User {{ name = {} }})", name);
        let params: HashMap<String, BoltType> = HashMap::new();
        params.insert("name", name.into());
        (query, params)
    }

    fn entity_from_node(node: Node) -> Neo4gEntity {
        let mut props: HashMap<String, BoltType> = HashMap::new();
        if let Ok(id) = node.get("id") {
            props.insert(("id", BoltType::from(id)));
        }
        if let Ok(name) = node.get("name") {
            props.insert(("name", BoltType::from(name)));
        }
        
        Neo4gEntity {
            entity_type: EntityType::Node,
            name: "User".to_string(),
            props,
        }
    }
}

pub enum UserProps {
    Id(i32),
    Name(String),
}

impl UserProps {
    pub fn to_query_param(prop: UserProps) -> BoltType {
        match prop {
            UserProps::Id(id) => BoltType::from(id),
            UserProps::Name(name) => BoltType::from(name),
        }
    }
}

pub struct Thing {
    id: ThingProps,
    test: ThingProps,
}

pub enum ThingProps {
    Id(i32),
    Test(String),
}

pub struct Group {
    id: GroupProps,
    name: GroupProps,
    something: GroupProps,
}

pub enum GroupProps {
    Id(i32),
    Name(String),
    Something(String),
}



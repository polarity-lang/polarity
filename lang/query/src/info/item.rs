// Item
//
//

#[derive(PartialEq, Eq, Clone)]
pub enum Item {
    Data(String),
    Codata(String),
    Def { name: String, type_name: String },
    Codef { name: String, type_name: String },
}

impl Item {
    pub fn type_name(&self) -> &str {
        match self {
            Item::Data(name) => name,
            Item::Codata(name) => name,
            Item::Def { type_name, .. } => type_name,
            Item::Codef { type_name, .. } => type_name,
        }
    }
}

use serde::{ Deserialize, Serialize };
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Institution {
    pub identifier: Uuid,
    pub name: String
}

impl Hash for Institution {
    fn hash<H: Hasher>(&self, f: &mut H) {
        self.identifier.hash(f);
    }
}

impl Institution {
    pub fn new(name: String) -> Self {
        Self {
            identifier: Uuid::new_v4(),
            name,
        }
    }

    pub fn find_best_match<'a>(institutions: &'a [Institution], text: &str) -> Option<&'a Institution> {
        if let Ok(id) = Uuid::parse_str(text) {
            return institutions.iter().find(|i| i.identifier == id);
        }

        institutions.iter().find(|i| i.name.to_lowercase().contains(&text.to_lowercase()))
    }
}

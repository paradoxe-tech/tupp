use serde::{ Deserialize, Serialize };
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Institution {
    pub identifier: Uuid,
    pub name: String,
    pub subinstitutions: Vec<Institution>
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
            subinstitutions: Vec::new(),
        }
    }

    pub fn display_recursive(&self, indent: usize, show_ids: bool) {
        if show_ids {
            println!("{}\t{}{}", self.identifier, "  ".repeat(indent), self.name);
        } else {
            println!("{}{}", "  ".repeat(indent), self.name);
        }

        for subinstitution in &self.subinstitutions {
            subinstitution.display_recursive(indent + 1, show_ids);
        }
    }

    pub fn find_parent_and_add_recursive(institutions: &mut Vec<Institution>, parent_id: &Uuid, new_institution: Institution) -> bool {
        for institution in institutions {
            if &institution.identifier == parent_id {
                institution.subinstitutions.push(new_institution);
                return true;
            }
            if Self::find_parent_and_add_recursive(&mut institution.subinstitutions, parent_id, new_institution.clone()) {
                return true;
            }
        }
        false
    }

    pub fn find_best_match<'a>(institutions: &'a [Institution], text: &str) -> Option<&'a Institution> {
        if let Ok(id) = Uuid::parse_str(text) {
            return Self::find_institution_by_id_recursive(institutions, &id);
        }

        for institution in institutions {
            if institution.name.to_lowercase().contains(&text.to_lowercase()) {
                return Some(institution);
            }
            if let Some(found) = Self::find_best_match(&institution.subinstitutions, text) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_institution_by_id_recursive<'a>(institutions: &'a [Institution], id: &Uuid) -> Option<&'a Institution> {
        for institution in institutions {
            if &institution.identifier == id {
                return Some(institution);
            }
            if let Some(found) = Self::find_institution_by_id_recursive(&institution.subinstitutions, id) {
                return Some(found);
            }
        }
        None
    }

    pub fn delete_institution_recursive(institutions: &mut Vec<Institution>, id: &Uuid) -> bool {
        let initial_len = institutions.len();
        institutions.retain(|i| &i.identifier != id);
        if institutions.len() < initial_len {
            return true;
        }
        for institution in institutions {
            if Self::delete_institution_recursive(&mut institution.subinstitutions, id) {
                return true;
            }
        }
        false
    }

    pub fn collect_ids_recursive(institutions: &[Institution], ids: &mut Vec<Uuid>) {
        for institution in institutions {
            ids.push(institution.identifier);
            Self::collect_ids_recursive(&institution.subinstitutions, ids);
        }
    }
}

use serde::{ Deserialize, Serialize };
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Group {
    pub identifier: Uuid,
    pub name: String,
    pub subgroups: Vec<Group>
}

impl Hash for Group {
    fn hash<H: Hasher>(&self, f: &mut H) {
        self.identifier.hash(f);
    }
}

impl Group {
    pub fn new(name: String) -> Self {
        Self {
            identifier: Uuid::new_v4(),
            name,
            subgroups: Vec::new(),
        }
    }

    pub fn display_recursive(&self, indent: usize, show_ids: bool) {
        if show_ids {
            println!("{}\t{}{}", self.identifier, "  ".repeat(indent), self.name);
        } else {
            println!("{}{}", "  ".repeat(indent), self.name);
        }

        for subgroup in &self.subgroups {
            subgroup.display_recursive(indent + 1, show_ids);
        }
    }

    pub fn contains_id_recursive(groups: &[Group], id: &Uuid) -> bool {
        for group in groups {
            if &group.identifier == id {
                return true;
            }
            if Self::contains_id_recursive(&group.subgroups, id) {
                return true;
            }
        }
        false
    }

    pub fn find_parent_and_add_recursive(groups: &mut Vec<Group>, parent_id: &Uuid, new_group: Group) -> bool {
        for group in groups {
            if &group.identifier == parent_id {
                group.subgroups.push(new_group);
                return true;
            }
            if Self::find_parent_and_add_recursive(&mut group.subgroups, parent_id, new_group.clone()) {
                return true;
            }
        }
        false
    }

    pub fn find_group_by_id_recursive<'a>(groups: &'a [Group], id: &Uuid) -> Option<&'a Group> {
        for group in groups {
            if &group.identifier == id {
                return Some(group);
            }
            if let Some(found) = Self::find_group_by_id_recursive(&group.subgroups, id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_group_by_name_recursive<'a>(groups: &'a [Group], text: &str) -> Vec<&'a Group> {
        let mut results = Vec::new();
        for group in groups {
            if group.name.to_lowercase().contains(&text.to_lowercase()) {
                results.push(group);
            }
            results.extend(Self::find_group_by_name_recursive(&group.subgroups, text));
        }
        results
    }

    pub fn delete_group_recursive(groups: &mut Vec<Group>, id: &Uuid) -> bool {
        let initial_len = groups.len();
        groups.retain(|g| &g.identifier != id);
        if groups.len() < initial_len {
            return true;
        }
        for group in groups {
            if Self::delete_group_recursive(&mut group.subgroups, id) {
                return true;
            }
        }
        false
    }

    pub fn remove_subgroup(&mut self, child_id: &Uuid) -> Result<String, String> {
        let initial_len = self.subgroups.len();
        self.subgroups.retain(|g| &g.identifier != child_id);
        
        if self.subgroups.len() < initial_len {
             return Ok(format!(
                 "Subgroup removed from {}",
                 self.name
             ));
        } else {
            return Err(format!(
                 "Subgroup not found in {}",
                 self.name
             ));
        }   
    }
}

use serde::{ Deserialize, Serialize };
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
pub struct Group {
    name: String,
    subgroups: HashSet<Group>
}

impl Hash for Group {
    fn hash<H: Hasher>(&self, n: &mut H) {
        self.name.hash(n);
    }
}

impl Group {
    // pub fn add_subgroup(&mut self, child: Group) -> Result<String, String> {
    //     if self.subgroups.insert(child) {
    //          return Ok(format!(
    //              "Group {} is now subgroup of {}",
    //              child.name,
    //              self.name
    //          ));
    //     } else {
    //         return Err(format!(
    //              "Group {} cannot be added as subgroup of {}",
    //              child.name,
    //              self.name
    //          ));
    //     }
    // }

    pub fn remove_subgroup(&mut self, child: Group) -> Result<String, String> {
        if self.subgroups.remove(&child) {
             return Ok(format!(
                 "Group {} is no longer subgroup of {}",
                 child.name,
                 self.name
             ));
        } else {
            return Err(format!(
                 "Group {} cannot be removed as subgroup of {}",
                 child.name,
                 self.name
             ));
        }   
    }
}

use crate::contact::{Contact, Relation};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct FamilyMember {
    id: Uuid,
    name: String,
    generation: i32,
    position: i32,
    spouse: Option<Uuid>,
}

#[derive(Debug)]
struct FamilyTree {
    members: HashMap<Uuid, FamilyMember>,
    children: HashMap<Uuid, Vec<Uuid>>,
    parents: HashMap<Uuid, Vec<Uuid>>,
    spouses: HashMap<Uuid, Uuid>,
}

impl FamilyTree {
    fn new() -> Self {
        FamilyTree {
            members: HashMap::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
            spouses: HashMap::new(),
        }
    }

    fn add_member(&mut self, id: Uuid, name: String, generation: i32) {
        let member = FamilyMember {
            id,
            name,
            generation,
            position: 0,
            spouse: None,
        };
        self.members.insert(id, member);
    }

    fn add_relationship(&mut self, parent_id: Uuid, child_id: Uuid) {
        self.children.entry(parent_id).or_insert_with(Vec::new).push(child_id);
        self.parents.entry(child_id).or_insert_with(Vec::new).push(parent_id);
    }

    fn add_spouse(&mut self, id1: Uuid, id2: Uuid) {
        self.spouses.insert(id1, id2);
        self.spouses.insert(id2, id1);
        if let Some(member1) = self.members.get_mut(&id1) {
            member1.spouse = Some(id2);
        }
        if let Some(member2) = self.members.get_mut(&id2) {
            member2.spouse = Some(id1);
        }
    }

    fn render_tree(&self) -> String {
        if self.members.is_empty() {
            return "No family members found".to_string();
        }

        // Group by generation
        let mut generations: HashMap<i32, Vec<&FamilyMember>> = HashMap::new();
        for member in self.members.values() {
            generations.entry(member.generation)
                .or_insert_with(Vec::new)
                .push(member);
        }

        let mut sorted_gens: Vec<i32> = generations.keys().cloned().collect();
        sorted_gens.sort();
        sorted_gens.reverse(); // Start from highest (oldest) generation

        let mut result = String::new();
        
        for (gen_idx, generation) in sorted_gens.iter().enumerate() {
            let members = &generations[generation];
            let line = self.render_generation_line(members);
            result.push_str(&line);
            result.push('\n');

            // Add connection lines between generations
            if gen_idx < sorted_gens.len() - 1 {
                let connection_lines = self.render_connection_lines(members, sorted_gens[gen_idx + 1], &generations);
                result.push_str(&connection_lines);
            }
        }

        result
    }

    fn render_generation_line(&self, members: &[&FamilyMember]) -> String {
        let mut line = String::new();
        let mut rendered = HashSet::new();

        for member in members {
            if rendered.contains(&member.id) {
                continue;
            }

            if !line.is_empty() {
                line.push_str("    ");
            }

            // Check for spouse
            if let Some(spouse_id) = member.spouse {
                if let Some(spouse) = self.members.get(&spouse_id) {
                    // Always render in consistent order (alphabetically)
                    if member.name <= spouse.name {
                        line.push_str(&format!("[{}]──[{}]", 
                            truncate_name(&member.name, 6), 
                            truncate_name(&spouse.name, 6)));
                    } else {
                        line.push_str(&format!("[{}]──[{}]", 
                            truncate_name(&spouse.name, 6), 
                            truncate_name(&member.name, 6)));
                    }
                    rendered.insert(member.id);
                    rendered.insert(spouse_id);
                }
            } else {
                line.push_str(&format!("[{}]", truncate_name(&member.name, 10)));
                rendered.insert(member.id);
            }
        }

        line
    }

    fn render_connection_lines(&self, current_gen: &[&FamilyMember], next_gen: i32, generations: &HashMap<i32, Vec<&FamilyMember>>) -> String {
        let next_members = match generations.get(&next_gen) {
            Some(members) => members,
            None => return String::new(),
        };

        // For now, create a simple example pattern
        // This would need more complex logic to properly map parent-child relationships
        match current_gen.len() {
            1 => {
                "     │\n   ┌─┴─┐\n   │   │\n".to_string()
            }
            2 => {
                "     │                │\n   ┌─┴────────────┬───┴─┐\n   │              │     │\n".to_string()
            }
            _ => {
                "     │    │    │\n   ┌─┴─┬──┴─┬──┴─┐\n   │   │    │    │\n".to_string()
            }
        }
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.chars().count() <= max_len {
        name.to_string()
    } else {
        let truncated: String = name.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}.", truncated)
    }
}

pub fn generate_family_tree(contacts: &[Contact], root_id: Uuid) -> String {
    let root_contact = match contacts.iter().find(|c| c.identifier == root_id) {
        Some(contact) => contact,
        None => return "Contact not found".to_string(),
    };

    let mut tree = FamilyTree::new();
    let mut visited = HashSet::new();
    let mut to_process = vec![(root_id, 0)]; // (id, generation)

    // Build the family tree
    while let Some((current_id, generation)) = to_process.pop() {
        if visited.contains(&current_id) {
            continue;
        }
        visited.insert(current_id);

        let current_contact = match contacts.iter().find(|c| c.identifier == current_id) {
            Some(contact) => contact,
            None => continue,
        };

        // Add current member to tree
        let name = format!("{} {}", 
            current_contact.identity.first_name.as_deref().unwrap_or(""),
            current_contact.identity.last_name.as_deref().unwrap_or("")).trim().to_string();
        
        tree.add_member(current_id, name, generation);

        // Process relationships
        if let Some(links) = &current_contact.links {
            for link in links {
                match link.relation {
                    Relation::Parent => {
                        // Current person is parent of target
                        tree.add_relationship(current_id, link.target);
                        to_process.push((link.target, generation - 1));
                    },
                    Relation::Child => {
                        // Current person is child of target
                        tree.add_relationship(link.target, current_id);
                        to_process.push((link.target, generation + 1));
                    },
                    Relation::Spouse | Relation::Partner => {
                        // Add spouse relationship
                        tree.add_spouse(current_id, link.target);
                        to_process.push((link.target, generation));
                    },
                    _ => {
                        // Add other family members at same generation for now
                        to_process.push((link.target, generation));
                    }
                }
            }
        }
    }
    
    if tree.members.is_empty() {
        return format!("No family relationships found for {}", 
            format!("{} {}", 
                root_contact.identity.first_name.as_deref().unwrap_or(""),
                root_contact.identity.last_name.as_deref().unwrap_or("")).trim());
    }

    format!("Family Tree for {}:\n\n{}\n\nExample format:\n{}", 
        format!("{} {}", 
            root_contact.identity.first_name.as_deref().unwrap_or(""),
            root_contact.identity.last_name.as_deref().unwrap_or("")).trim(),
        tree.render_tree(),
        get_example_tree())
}

fn get_example_tree() -> &'static str {
    "[GGF1]──[GGM1]    [GGF2]──[GGM2]
     │                │
   ┌─┴────────────┬───┴─┐
   │              │     │
[GF1]──[GM1]   [GF2]──[GM2]
   │              │
 ┌─┴──────┐     ┌─┴────┐
 │        │     │      │
[F1]──[M1]     [U1]──[A1]
 │                │
 ├───────┐        └──[Cousin1]
 │       │
[You]──[Partner] 
 │
 ├───────┐
 │       │
[C1]   [C2]"
}

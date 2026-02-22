use crate::cli::GroupCommand;
use crate::models::TuppData;
use crate::group::Group;
use crate::error::TuppError;
use crate::storage::save_data;
use uuid::Uuid;
use std::path::PathBuf;

pub fn handle_group_command(
    command: GroupCommand,
    data: &mut TuppData,
    file_path: &PathBuf,
) -> Result<(), TuppError> {
    match command {
        GroupCommand::List { show_ids } => {
            if data.groups.is_empty() {
                println!("No groups found.");
            } else {
                for group in &data.groups {
                    group.display_recursive(0, show_ids);
                }
            }
        }
        GroupCommand::New { name, parent } => {
            let new_group = Group::new(name);
            
            if let Some(parent_str) = parent {
                let parent_id = find_group_best_match(&data.groups, &parent_str)
                    .map(|g| g.identifier);

                if let Some(id) = parent_id {
                    if Group::find_parent_and_add_recursive(&mut data.groups, &id, new_group.clone()) {
                        println!("Subgroup created with ID: {}", new_group.identifier);
                    } else {
                        println!("Failed to add subgroup.");
                    }
                } else {
                    println!("Parent group matching '{}' not found.", parent_str);
                    return Ok(());
                }
            } else {
                println!("Group created with ID: {}", new_group.identifier);
                data.groups.push(new_group);
            }
            save_data(file_path, data)?;
        }
        GroupCommand::Del { id } => {
            let id_uuid = if let Some(group) = find_group_best_match(&data.groups, &id) {
                group.identifier
            } else {
                println!("No group found matching '{}'.", id);
                return Ok(());
            };

            if Group::delete_group_recursive(&mut data.groups, &id_uuid) {
                println!("Group deleted successfully.");
                // Optional: remove this group from all contacts
                for contact in &mut data.contacts {
                    if let Some(ref mut contact_groups) = contact.groups {
                        contact_groups.remove(&id_uuid);
                    }
                }
                save_data(file_path, data)?;
            } else {
                println!("Group not found.");
            }
        }
        GroupCommand::Find { text } => {
            if let Some(group) = find_group_best_match(&data.groups, &text) {
                println!("{} ({})", group.name, group.identifier);
            } else {
                println!("No group found matching '{}'.", text);
            }
        }
        GroupCommand::Show { id } => {
            if let Some(group) = find_group_best_match(&data.groups, &id) {
                println!("Group: {}", group.name);
                println!("ID: {}", group.identifier);
                
                let members: Vec<_> = data.contacts.iter()
                    .filter(|c| c.groups.as_ref().map_or(false, |g| g.contains(&group.identifier)))
                    .collect();
                
                if members.is_empty() {
                    println!("Members: None");
                } else {
                    println!("Members:");
                    for m in members {
                        println!("  - {} ({})", m.format_name("FIRST LAST"), m.identifier);
                    }
                }
            } else {
                println!("Group not found matching '{}'.", id);
            }
        }
    }
    Ok(())
}

fn find_group_best_match<'a>(groups: &'a [Group], text: &str) -> Option<&'a Group> {
    Group::find_best_match(groups, text)
}

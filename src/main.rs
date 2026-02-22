mod error;
mod models;
mod contact;
mod group;
mod storage;
mod sanitize;
mod interactions;

use crate::storage::*;
use clap::{Parser, Subcommand};
use crate::contact::Contact;
use crate::group::Group;
use crate::error::TuppError;
use uuid::Uuid;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "tupp", version = "1.2.1", author = "mtripnaux & gaiadrd")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage contacts.
    Contact {
        #[clap(subcommand)]
        command: ContactCommand,
    },

    /// Manage groups.
    Group {
        #[clap(subcommand)]
        command: GroupCommand,
    },

    /// Export contacts to a specified file.
    Export {
        /// The path to the export file.
        path: String,
    },

    /// Initialize the contact list (clears all data).
    Init,

    /// Show the path to the data file.
    Where,
}

#[derive(Subcommand, Debug)]
enum ContactCommand {
    /// List all contacts.
    List {
        /// Display pattern for contact names (e.g., "TITLE FIRST LAST").
        #[clap(short, long, default_value = "TITLE FIRST LAST")]
        pattern: String,

        /// Show contact IDs in the output.
        #[clap(short = 'i', long)]
        show_ids: bool,
    },

    /// Register a new contact.
    New {
        /// The title of the contact.
        #[clap(short = 't', long)]
        title: Option<String>,
        /// The first name of the contact.
        #[clap(short = 'f', long)]
        first_name: Option<String>,
        /// The middle name of the contact.
        #[clap(short = 'm', long)]
        middle_name: Option<String>,
        /// The last name of the contact.
        #[clap(short = 'l', long)]
        last_name: Option<String>,
        /// The post-nominal title of the contact.
        #[clap(short = 'p', long)]
        post_nominal: Option<String>,
        /// The gender of the contact.
        #[clap(short = 'g', long)]
        gender: Option<String>,
    },

    /// Delete a contact by its ID.
    Del {
        /// The ID of the contact to delete.
        id: String,
    },

    /// Find a contact by searching for text in their details.
    Find {
        /// The text to search for in contact details.
        text: String,
    },

    /// Show detailed information for a specific contact.
    Show {
        /// The ID of the contact to display.
        id: String,
    },

    /// Add information to an existing contact.
    Add {
        /// The ID of the contact to modify.
        id: String,
        /// The type of information to add.
        #[clap(subcommand)]
        add_type: AddType,
    },
}

#[derive(Subcommand, Debug)]
enum GroupCommand {
    /// List all groups.
    List {
        /// Show group IDs in the output.
        #[clap(short = 'i', long)]
        show_ids: bool,
    },
    /// Create a new group.
    New {
        /// The name of the group.
        name: String,
        /// The ID of the parent group (optional).
        #[clap(short, long)]
        parent: Option<String>,
    },
    /// Delete a group by its ID.
    Del {
        /// The ID of the group to delete.
        id: String,
    },
    /// Find a group by searching for text in their name.
    Find {
        /// The text to search for in group names.
        text: String,
    },
    /// Show detailed information for a specific group.
    Show {
        /// The ID of the group to display.
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum AddType {
    /// Add a social media account.
    Social {
        #[clap(short = 'n', long)]
        network: Option<String>,
        #[clap(short = 'u', long)]
        username: Option<String>,
    },
    /// Add birth information.
    Birth {
        #[clap(short = 'f', long)]
        first_name: Option<String>,
        #[clap(short = 'm', long)]
        middle_name: Option<String>,
        #[clap(short = 'l', long)]
        last_name: Option<String>,
        #[clap(short = 'd', long)]
        day: Option<u8>,
        #[clap(short = 'M', long)]
        month: Option<u8>,
        #[clap(short = 'y', long)]
        year: Option<i32>,
    },
    /// Add death information.
    Death {
        #[clap(short = 'd', long)]
        day: Option<u8>,
        #[clap(short = 'M', long)]
        month: Option<u8>,
        #[clap(short = 'y', long)]
        year: Option<i32>,
    },
    /// Add gender information.
    Gender {
        #[clap(short = 'g', long)]
        gender: Option<String>,
    },
    /// Add an email address.
    Email {
        #[clap(short = 'l', long)]
        label: Option<String>,
        #[clap(short = 'a', long)]
        address: Option<String>,
    },
    /// Add a phone number.
    Phone {
        #[clap(short = 'l', long)]
        label: Option<String>,
        #[clap(short = 'i', long)]
        indicator: Option<u16>,
        #[clap(short = 'n', long)]
        number: Option<u32>,
    },
    /// Add contact to a group.
    Group {
        /// The name or ID of the group.
        name_or_id: String,
    },
    /// Link to another contact.
    Link {
        /// The ID of the contact to link to.
        other_id: String,
        /// The type of relationship.
        relation_type: String,
    },
    /// Add an address.
    Address {
        #[clap(short = 'l', long)]
        label: Option<String>,
        #[clap(short = 'c', long)]
        country: Option<String>,
        #[clap(short = 'r', long)]
        region: Option<String>,
        #[clap(short = 'i', long)]
        city: Option<String>,
        #[clap(short = 'p', long)]
        post_code: Option<String>,
        #[clap(short = 's', long)]
        street: Option<String>,
        #[clap(short = 'n', long)]
        number: Option<String>,
    },
}

fn find_best_match<'a>(contacts: &'a [Contact], text: &str) -> Option<&'a Contact> {
    if let Ok(id) = Uuid::parse_str(text) {
        return contacts.iter().find(|c| c.identifier == id);
    }

    let closure_score = |contact: &Contact| -> i32 {
        let name_score = if contact
            .format_name("TITLE FIRST MIDDLE LAST POST")
            .to_lowercase()
            .contains(
                &crate::sanitize::trim_extra_spaces(text)
                .to_lowercase()
            ) { 1 } else { 0 };
            
        return name_score
    };
    
    let best_match = contacts
        .iter()
        .max_by_key(|contact| { closure_score(contact) })?;

    if closure_score(best_match) > 0 {
        return Some(best_match);
    } else { return None };
}

fn find_group_best_match<'a>(groups: &'a [Group], text: &str) -> Option<&'a Group> {
    if let Ok(id) = Uuid::parse_str(text) {
        return Group::find_group_by_id_recursive(groups, &id);
    }
    
    for group in groups {
        if group.name.to_lowercase().contains(&text.to_lowercase()) {
            return Some(group);
        }
        if let Some(found) = find_group_best_match(&group.subgroups, text) {
            return Some(found);
        }
    }
    None
}

fn main() -> Result<(), TuppError> {
    let cli = Cli::parse();

    let contacts_file = ensure_config_file()?;
    let mut data = load_data(&contacts_file)?;
    
    match cli.command {
        Commands::Contact { command } => {
            match command {
                ContactCommand::List { pattern, show_ids } => {
                    for contact in &data.contacts {
                        if show_ids {
                            println!("{}\t{}", contact.identifier, contact.format_name(&pattern));
                        } else {
                            println!("{}", contact.format_name(&pattern));
                        }
                    }
                },
                ContactCommand::New { title, first_name, middle_name, last_name, post_nominal, gender } => {
                    let new_contact = interactions::create_contact_interactive(title, first_name, middle_name, last_name, post_nominal, gender)?;
                    data.contacts.push(new_contact);

                    save_data(&contacts_file, &data)?;

                    println!("Contact added successfully!");
                },
                ContactCommand::Del { id } => {
                    let id_uuid = if let Some(contact) = find_best_match(&data.contacts, &id) {
                        contact.identifier
                    } else {
                        println!("No contact found matching '{}'.", id);
                        return Ok(());
                    };
                    
                    let initial_len = data.contacts.len();
                    data.contacts.retain(|contact| contact.identifier != id_uuid);

                    if data.contacts.len() < initial_len {
                        save_data(&contacts_file, &data)?;
                        println!("Contact has been deleted.");
                    } else {
                        println!("No contact found with this id.");
                    }
                },
                ContactCommand::Find { text } => {
                    if let Some(contact) = find_best_match(&data.contacts, &text) {
                        println!("{}", contact.identifier);
                    } else {
                        println!("No contact found matching '{}'.", text);
                    }
                },
                ContactCommand::Show { id } => {
                    if let Some(contact) = find_best_match(&data.contacts, &id) {
                        println!("{}", contact);
                    } else {
                        println!("No contact found matching '{}'.", id);
                    }
                },
                ContactCommand::Add { id, add_type } => {
                    let contact_identifier = if let Some(contact) = find_best_match(&data.contacts, &id) {
                        contact.identifier
                    } else {
                        println!("No contact found matching '{}'.", id);
                        return Ok(());
                    };
                    
                    // Check for duplicate for the type being added
                    if let Some(contact) = find_best_match(&data.contacts, &id) {
                        match &add_type {
                            AddType::Email { label, .. } => {
                                let label_str = label.clone().unwrap_or_else(|| "default".to_string());
                                if let Some(emails) = &contact.emails {
                                    if emails.iter().any(|e| e.label.as_deref() == Some(&label_str)) {
                                        eprintln!("Error: Duplicate email label '{}'.", label_str);
                                        return Ok(());
                                    }
                                }
                            },
                            AddType::Phone { label, .. } => {
                                let label_str = label.clone().unwrap_or_else(|| "default".to_string());
                                if let Some(phones) = &contact.phones {
                                    if phones.iter().any(|p| p.label.as_deref() == Some(&label_str)) {
                                        return Err(TuppError::Duplicate(format!("Phone label '{}' already exists", label_str)));
                                    }
                                }
                            },
                            AddType::Address { label, .. } => {
                                let label_str = label.clone().unwrap_or_else(|| "default".to_string());
                                if let Some(address) = &contact.address {
                                    if address.label.as_deref() == Some(&label_str) {
                                        return Err(TuppError::Duplicate(format!("Address label '{}' already exists", label_str)));
                                    }
                                }
                            },
                            AddType::Social { network, .. } => {
                                if let Some(network) = network {
                                    if let Some(socials) = &contact.socials {
                                        if socials.iter().any(|s| &s.network == network) {
                                            return Err(TuppError::Duplicate(format!("Social network '{}' already exists", network)));
                                        }
                                    }
                                }
                            },
                            AddType::Group { name_or_id } => {
                                if let Some(groups) = &contact.groups {
                                    if let Some(group) = find_group_best_match(&data.groups, &name_or_id) {
                                        if groups.contains(&group.identifier) {
                                            return Err(TuppError::Duplicate(format!("Contact already in group '{}'", group.name)));
                                        }
                                    }
                                }
                            },
                            AddType::Link { other_id, .. } => {
                                if let Some(other) = find_best_match(&data.contacts, &other_id) {
                                    if let Some(links) = &contact.links {
                                        if links.iter().any(|l| l.target == other.identifier) {
                                            return Err(TuppError::Duplicate("Link to this contact already exists".to_string()));
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    // Handle link case separately to avoid borrowing issues
                    if let AddType::Link { other_id, relation_type } = &add_type {
                        let other_identifier = if let Some(contact) = find_best_match(&data.contacts, other_id) {
                            contact.identifier
                        } else {
                            println!("No contact found matching '{}'.", other_id);
                            return Ok(());
                        };
                        
                        // Find both contacts and create bidirectional link
                        let mut contact_a_index = None;
                        let mut contact_b_index = None;
                        
                        for (index, contact) in data.contacts.iter().enumerate() {
                            if contact.identifier == contact_identifier {
                                contact_a_index = Some(index);
                            }
                            if contact.identifier == other_identifier {
                                contact_b_index = Some(index);
                            }
                        }
                        
                        if let (Some(a_idx), Some(b_idx)) = (contact_a_index, contact_b_index) {
                            let (contact_a, contact_b) = if a_idx < b_idx {
                                let (left, right) = data.contacts.split_at_mut(b_idx);
                                (&mut left[a_idx], &mut right[0])
                            } else {
                                let (left, right) = data.contacts.split_at_mut(a_idx);
                                (&mut right[0], &mut left[b_idx])
                            };
                            
                            if let Err(error) = Contact::create_bidirectional_link(contact_a, contact_b, relation_type.clone()) {
                                println!("{}", error);
                                return Ok(());
                            }
                        }
                    } else {
                        // Handle other add types
                        if let Some(contact) = data.contacts.iter_mut().find(|c| c.identifier == contact_identifier) {
                            match add_type {
                                AddType::Social { network, username } => interactions::add_social_to_contact(contact, network, username),
                                AddType::Birth { first_name, middle_name, last_name, day, month, year } => interactions::add_birth_to_contact(contact, first_name, middle_name, last_name, day, month, year),
                                AddType::Death { day, month, year } => interactions::add_death_to_contact(contact, day, month, year),
                                AddType::Gender { gender } => interactions::add_gender_to_contact(contact, gender),
                                AddType::Email { label, address } => {
                                    let success = interactions::add_email_to_contact(contact, label, address);
                                    if !success {
                                        return Ok(());
                                    }
                                },
                                AddType::Phone { label, indicator, number } => {
                                    let success = interactions::add_phone_to_contact(contact, label, indicator, number);
                                    if !success {
                                        return Ok(());
                                    }
                                },
                                AddType::Group { name_or_id } => {
                                    if let Some(group) = find_group_best_match(&data.groups, &name_or_id) {
                                        if contact.groups.is_none() {
                                            contact.groups = Some(std::collections::HashSet::new());
                                        }
                                        contact.groups.as_mut().unwrap().insert(group.identifier);
                                        println!("Contact added to group '{}'.", group.name);
                                    } else {
                                        println!("No group found matching '{}'.", name_or_id);
                                        return Ok(());
                                    }
                                }
                                AddType::Link { .. } => unreachable!(), // Already handled above
                                AddType::Address { label, country, region, city, post_code, street, number } => {
                                    let success = interactions::address::add_address_to_contact(contact, label, country, region, city, post_code, street, number);
                                    if !success {
                                        return Ok(());
                                    }
                                },
                            }
                        }
                    }
                    
                    save_data(&contacts_file, &data)?;
                    println!("Information added successfully!");
                },
            }
        },
        Commands::Group { command } => {
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
            save_data(&contacts_file, &data)?;
        },
        Commands::Export { path } => { 
            save_data(&PathBuf::from(path), &data)?
        },
        Commands::Init => {
            fs::write(&contacts_file, "{\"contacts\": [], \"groups\": []}")
                .expect("Failed to write empty contact file");
        },
        Commands::Where => {
            println!("{}", contacts_file.display());
        },
    }

    Ok(())
}
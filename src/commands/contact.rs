use crate::cli::{ContactCommand, AddType};
use crate::models::TuppData;
use crate::contact::Contact;
use crate::group::Group;
use crate::interactions;
use crate::error::TuppError;
use crate::storage::save_data;
use uuid::Uuid;
use std::path::PathBuf;

pub fn handle_contact_command(
    command: ContactCommand,
    data: &mut TuppData,
    file_path: &PathBuf,
) -> Result<(), TuppError> {
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

            save_data(file_path, data)?;

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
                save_data(file_path, data)?;
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
                                return Err(TuppError::Duplicate(format!("Email label '{}' already exists", label_str)));
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
                    AddType::Social { label, .. } => {
                        let label_str = label.clone().unwrap_or_else(|| "default".to_string());
                        if let Some(socials) = &contact.socials {
                            if socials.iter().any(|s| s.label.as_deref() == Some(&label_str)) {
                                return Err(TuppError::Duplicate(format!("Social label '{}' already exists", label_str)));
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
                        AddType::Social { label, network, username } => {
                             let success = interactions::add_social_to_contact(contact, label, network, username);
                            if !success {
                                return Ok(());
                            }
                        },
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
            
            save_data(file_path, data)?;
            println!("Information added successfully!");
        },
    }
    Ok(())
}

fn find_best_match<'a>(contacts: &'a [Contact], text: &str) -> Option<&'a Contact> {
    Contact::find_best_match(contacts, text)
}

fn find_group_best_match<'a>(groups: &'a [Group], text: &str) -> Option<&'a Group> {
    Group::find_best_match(groups, text)
}


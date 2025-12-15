mod unwrap;
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
use uuid::Uuid;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "tupp", version = "1.1.0", author = "paradoxe-tech & floriandrd")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
    New,

    /// Delete a contact by its ID.
    Del {
        /// The ID of the contact to delete.
        id: String,
    },

    /// Initialize the contact list (clears all data).
    Init,

    /// Export contacts to a specified file.
    Export {
        /// The path to the export file.
        path: String,
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

    /// Find a contact then show detailed information for it.
    Display {
        /// The text to search for in contact details.
        text: String,
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
enum AddType {
    /// Add a social media account.
    Social,
    /// Add birth information.
    Birth,
    /// Add death information.
    Death,
    /// Add gender information.
    Gender,
    /// Add an email address.
    Email,
    /// Add a phone number.
    Phone,
    /// Link to another contact.
    Link {
        /// The ID of the contact to link to.
        other_id: String,
        /// The type of relationship.
        relation_type: String,
    },
}

fn find_best_match<'a>(contacts: &'a [Contact], text: &str) -> Option<&'a Contact> {
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
        .max_by_key(|contact| { closure_score(contact) });

    if closure_score(best_match?) > 0 {
        return Some(best_match?);
    } else { return None };
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let contacts_file = ensure_config_file()?;
    let mut contacts = load_contacts(&contacts_file)?;
    
    match cli.command {
        Commands::Init => {
            fs::write(&contacts_file, "[]")
                .expect("Failed to write empty contact file");
        },
        Commands::Export { path } => { 
            save_contacts(&PathBuf::from(path), &contacts)?
        },
        Commands::List { pattern, show_ids } => {
            for contact in contacts {
                if show_ids {
                    println!("{}\t{}", contact.identifier, contact.format_name(&pattern));
                } else {
                    println!("{}", contact.format_name(&pattern));
                }
            }
        },
        Commands::New => {
            let new_contact = Contact::new_from_input();
            contacts.push(new_contact);

            save_contacts(&contacts_file, &contacts)?;

            println!("Contact added successfully!");
        },
        Commands::Del { id } => {
            let id_uuid = match Uuid::parse_str(&id) {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    println!("Invalid ID format. Please provide a valid UUID.");
                    return Ok(());
                }
            };
            
            let initial_len = contacts.len();
            contacts.retain(|contact| contact.identifier != id_uuid);

            if contacts.len() < initial_len {
                save_contacts(&contacts_file, &contacts)?;

                println!("Contact has been deleted.");
            } else {
                println!("No contact found with this id.");
            }
        },
        Commands::Find { text } => {
            if let Some(contact) = find_best_match(&contacts, &text) {
                println!("{}", contact.identifier);
            } else {
                println!("No contact found matching '{}'.", text);
            }
        },
        Commands::Show { id } => {
            let id_uuid = match Uuid::parse_str(&id) {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    println!("Invalid ID format. Please provide a valid UUID.");
                    return Ok(());
                }
            };

            if let Some(contact) = contacts.iter().find(|c| c.identifier == id_uuid) {
                println!("{}", contact);
            } else {
                println!("No contact found with ID: {}", id);
            }
        },
        Commands::Display { text } => {
            if let Some(contact) = find_best_match(&contacts, &text) {
                println!("{contact}");
            } else {
                println!("No contact found matching '{text}'.");
            }
        },
        Commands::Add { id, add_type } => {
            let id_uuid = match Uuid::parse_str(&id) {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    println!("Invalid ID format. Please provide a valid UUID.");
                    return Ok(());
                }
            };

            // Handle link case separately to avoid borrowing issues
            if let AddType::Link { other_id, relation_type } = &add_type {
                let other_uuid = match Uuid::parse_str(other_id) {
                    Ok(parsed_id) => parsed_id,
                    Err(_) => {
                        println!("Invalid other ID format. Please provide a valid UUID.");
                        return Ok(());
                    }
                };
                
                // Check if both contacts exist
                let contact_exists = contacts.iter().any(|c| c.identifier == id_uuid);
                let other_exists = contacts.iter().any(|c| c.identifier == other_uuid);
                
                if !contact_exists {
                    println!("No contact found with ID: {}", id);
                    return Ok(());
                }
                
                if !other_exists {
                    println!("No contact found with ID: {}", other_id);
                    return Ok(());
                }
                
                // Find both contacts and create bidirectional link
                let mut contact_a_index = None;
                let mut contact_b_index = None;
                
                for (index, contact) in contacts.iter().enumerate() {
                    if contact.identifier == id_uuid {
                        contact_a_index = Some(index);
                    }
                    if contact.identifier == other_uuid {
                        contact_b_index = Some(index);
                    }
                }
                
                if let (Some(a_idx), Some(b_idx)) = (contact_a_index, contact_b_index) {
                    let (contact_a, contact_b) = if a_idx < b_idx {
                        let (left, right) = contacts.split_at_mut(b_idx);
                        (&mut left[a_idx], &mut right[0])
                    } else {
                        let (left, right) = contacts.split_at_mut(a_idx);
                        (&mut right[0], &mut left[b_idx])
                    };
                    
                    if let Err(error) = Contact::create_bidirectional_link(contact_a, contact_b, relation_type.clone()) {
                        println!("{}", error);
                        return Ok(());
                    }
                }
            } else {
                // Handle other add types
                if let Some(contact) = contacts.iter_mut().find(|c| c.identifier == id_uuid) {
                    match add_type {
                        AddType::Social => contact.add_social_interactive(),
                        AddType::Birth => contact.add_birth_interactive(),
                        AddType::Death => contact.add_death_interactive(),
                        AddType::Gender => contact.add_gender_interactive(),
                        AddType::Email => contact.add_email_interactive(),
                        AddType::Phone => contact.add_phone_interactive(),
                        AddType::Link { .. } => unreachable!(), // Already handled above
                    }
                } else {
                    println!("No contact found with ID: {}", id);
                    return Ok(());
                }
            }
            
            save_contacts(&contacts_file, &contacts)?;
            println!("Information added successfully!");
        },
        
    }

    Ok(())
}
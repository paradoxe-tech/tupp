mod unwrap;
mod models;
mod contact;
mod group;
mod storage;
mod sanitize;

use crate::storage::*;
use clap::{Parser, Subcommand};
use crate::contact::Contact;
use crate::group::Group;
use uuid::Uuid;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "tupp", version = "1.0", author = "paradoxe-tech & floriandrd")]
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

    /// Add a new contact.
    Add,

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
        Commands::Add => {
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
    }

    Ok(())
}
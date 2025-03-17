mod unwrap;
mod models;
mod contact;
mod group;

use clap::{Parser, Subcommand};
use std::fs;
use crate::contact::Contact;
use crate::group::Group;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[clap(name = "tup", version = "1.0", author = "paradoxe-tech")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List {
        #[clap(short, long, default_value = "TITLE FIRST LAST")]
        pattern: String,
    },
    Add,
    Del {
        id: String
    }
}

fn main() {
    let cli = Cli::parse();
    
    let json_data = fs::read_to_string("sample.json")
        .expect("Failed to read the JSON file");

    let mut contacts: Vec<Contact> = serde_json::from_str(&json_data)
        .unwrap_or_else(|_| Vec::new());
    
    match cli.command {
        Commands::List { pattern } => {
            for contact in contacts {
                println!("{}\t{}", contact.identifier, contact.format_name(&pattern));
            }
        },
        Commands::Add => {
            let new_contact = Contact::new_from_input();
            contacts.push(new_contact);

            fs::write("sample.json", serde_json::to_string_pretty(&contacts).unwrap())
                .expect("Failed to save contacts");

            println!("Contact added successfully!");
        },
        Commands::Del { id } => {
            
            let id_uuid = match Uuid::parse_str(&id) {
                Ok(parsed_id) => parsed_id,
                Err(_) => {
                    println!("Invalid ID format. Please provide a valid UUID.");
                    return;
                }
            };
            
            let initial_len = contacts.len();
            contacts.retain(|contact| contact.identifier != id_uuid);

            if contacts.len() < initial_len {
                fs::write("sample.json", serde_json::to_string_pretty(&contacts).unwrap())
                    .expect("Failed to save contacts");

                println!("Contact has been deleted.");
            } else {
                println!("No contact found with this id.");
            }
        },
    }
}
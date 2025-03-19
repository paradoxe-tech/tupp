mod unwrap;
mod models;
mod contact;
mod group;

use clap::{Parser, Subcommand};
use crate::contact::Contact;
use crate::group::Group;
use uuid::Uuid;
use dirs;

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

fn get_config_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("Failed to get home directory");
    path.push(".config");
    path.push("tupp");
    path
}

fn ensure_config_file() -> io::Result<PathBuf> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let contacts_file = config_dir.join("contacts.json");
    if !contacts_file.exists() {
        let mut file = File::create(&contacts_file)?;
        writeln!(file, "[]")?;
    }

    Ok(contacts_file)
}

fn load_contacts(path: &PathBuf) -> io::Result<Vec<Contact>> {
    let data = fs::read_to_string(path)?;
    let contacts: Vec<Contact> = serde_json::from_str(&data)?;
    Ok(contacts)
}

fn save_contacts(path: &PathBuf, contacts: &Vec<Contact>) -> io::Result<()> {
    let data = serde_json::to_string_pretty(contacts)?;
    fs::write(path, data)
}

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
    },
    Init,
    Export {
        path: String
    }
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
        Commands::List { pattern } => {
            for contact in contacts {
                println!("{}\t{}", contact.identifier, contact.format_name(&pattern));
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
    }

    Ok(())
}
use crate::cli::InstitutionCommand;
use crate::models::TuppData;
use crate::institution::Institution;
use crate::error::TuppError;
use crate::storage::save_data;
use std::path::PathBuf;

pub fn handle_institution_command(
    command: InstitutionCommand,
    data: &mut TuppData,
    file_path: &PathBuf,
) -> Result<(), TuppError> {
    match command {
        InstitutionCommand::List { show_ids } => {
            if data.institutions.is_empty() {
                println!("No institutions found.");
            } else {
                for institution in &data.institutions {
                    if show_ids {
                        println!("{}\t{}", institution.identifier, institution.name);
                    } else {
                        println!("{}", institution.name);
                    }
                }
            }
        }
        InstitutionCommand::New { name } => {
            let new_institution = Institution::new(name);
            println!("Institution created with ID: {}", new_institution.identifier);
            data.institutions.push(new_institution);
            save_data(file_path, data)?;
        }
        InstitutionCommand::Del { id } => {
            let id_uuid = if let Some(institution) = find_best_match(&data.institutions, &id) {
                institution.identifier
            } else {
                println!("No institution found matching '{}'.", id);
                return Ok(());
            };

            let initial_len = data.institutions.len();
            data.institutions.retain(|i| i.identifier != id_uuid);

            if data.institutions.len() < initial_len {
                for contact in &mut data.contacts {
                    if let Some(ref mut positions) = contact.positions {
                        positions.retain(|p| p.institution != id_uuid);
                    }
                }
                save_data(file_path, data)?;
                println!("Institution deleted successfully.");
            } else {
                println!("Institution not found.");
            }
        }
        InstitutionCommand::Find { text } => {
            if let Some(institution) = find_best_match(&data.institutions, &text) {
                println!("{} ({})", institution.name, institution.identifier);
            } else {
                println!("No institution found matching '{}'.", text);
            }
        }
        InstitutionCommand::Show { id } => {
            if let Some(institution) = find_best_match(&data.institutions, &id) {
                println!("Institution: {}", institution.name);
                println!("ID: {}", institution.identifier);

                let members: Vec<_> = data.contacts.iter()
                    .filter_map(|c| {
                        c.positions.as_ref()
                            .and_then(|ps| ps.iter().find(|p| p.institution == institution.identifier))
                            .map(|p| (c, p))
                    })
                    .collect();

                if members.is_empty() {
                    println!("Members: None");
                } else {
                    println!("Members:");
                    for (contact, position) in members {
                        println!("  - {} : {} ({})", contact.format_name("FIRST LAST"), position.title, position.tenure);
                    }
                }
            } else {
                println!("Institution not found matching '{}'.", id);
            }
        }
    }
    Ok(())
}

fn find_best_match<'a>(institutions: &'a [Institution], text: &str) -> Option<&'a Institution> {
    Institution::find_best_match(institutions, text)
}

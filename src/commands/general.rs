use crate::cli::Commands;
use crate::models::TuppData;
use crate::storage::save_data;
use crate::error::TuppError;
use std::path::PathBuf;
use std::fs;

pub fn handle_general_command(
    command: Commands,
    data: &mut TuppData,
    file_path: &PathBuf,
) -> Result<(), TuppError> {
    match command {
        Commands::Export { path } => { 
            save_data(&PathBuf::from(path), data)?
        },
        Commands::Init => {
            fs::write(file_path, "{\"contacts\": [], \"groups\": []}")
                .expect("Failed to write empty contact file");
        },
        Commands::Where => {
            println!("{}", file_path.display());
        },
        Commands::Validate { path } => {
            let p = path.map(PathBuf::from).unwrap_or(file_path.clone());
            println!("Validating {}...", p.display());
            crate::validation::validate_file(&p)?;
            println!("Validation successful.");
        },
        _ => {}
    }
    Ok(())
}

mod cli;
mod error;
mod models;
mod contact;
mod group;
mod storage;
mod sanitize;
mod interactions;
mod commands;

use crate::storage::*;
use clap::Parser;
use crate::cli::{Cli, Commands};
use crate::error::TuppError;

fn main() -> Result<(), TuppError> {
    let cli = Cli::parse();

    let contacts_file = ensure_config_file()?;
    let mut data = load_data(&contacts_file)?;
    
    match cli.command {
        Commands::Contact { command } => {
            commands::handle_contact_command(command, &mut data, &contacts_file)?;
        },
        Commands::Group { command } => {
            commands::handle_group_command(command, &mut data, &contacts_file)?;
        },
        other_command => {
            commands::handle_general_command(other_command, &mut data, &contacts_file)?;
        },
    }

    Ok(())
}

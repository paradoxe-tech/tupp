use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "tupp", version = "1.2.2", author = "mtripnaux & gaiadrd")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
pub enum ContactCommand {
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
pub enum GroupCommand {
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
pub enum AddType {
    /// Add a social media account.
    Social {
        #[clap(short = 'l', long)]
        label: Option<String>,
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

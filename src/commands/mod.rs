pub mod contact;
pub mod group;
pub mod institution;
pub mod general;
pub mod serve;
pub mod sync;

pub use contact::handle_contact_command;
pub use group::handle_group_command;
pub use institution::handle_institution_command;
pub use general::handle_general_command;
pub use serve::handle_serve_command;
pub use sync::handle_sync_command;

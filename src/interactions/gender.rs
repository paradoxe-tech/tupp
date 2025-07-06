use dialoguer::Select;
use crate::models::Gender;

pub fn create_gender_interactive() -> Gender {
    let options = vec!["Male", "Female", "Non-binary"];
    
    let selection = Select::new()
        .with_prompt("Select gender")
        .default(0)
        .items(&options)
        .interact()
        .unwrap();

    match selection {
        0 => Gender::Male,
        1 => Gender::Female,
        2 => Gender::NonBinary,
        _ => Gender::Male, // Default fallback
    }
}

pub fn add_gender_to_contact(contact: &mut crate::contact::Contact) {
    contact.identity.gender = Some(create_gender_interactive());
}

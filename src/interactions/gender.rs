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

pub fn add_gender_to_contact(contact: &mut crate::contact::Contact, gender: Option<String>) {
    if let Some(gender_str) = gender {
        let gender = match gender_str.to_lowercase().as_str() {
            "male" => Some(Gender::Male),
            "female" => Some(Gender::Female),
            "non-binary" | "nonbinary" => Some(Gender::NonBinary),
            _ => None,
        };
        if let Some(g) = gender {
            contact.identity.gender = Some(g);
            return;
        } else {
            println!("Invalid gender. Valid values are: male, female, non-binary.");
        }
    }
    contact.identity.gender = Some(create_gender_interactive());
}

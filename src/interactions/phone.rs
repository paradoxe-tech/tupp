use dialoguer::{Input, Confirm};
use crate::models::PhoneNumber;

pub fn create_phone_interactive(existing_labels: &[String]) -> PhoneNumber {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this phone number?")
        .default(false)
        .interact()
        .unwrap()
    {
        loop {
            let input: String = Input::new()
                .with_prompt("Phone Label (e.g., Work, Mobile, Home)")
                .interact_text()
                .unwrap();
            
            if existing_labels.contains(&input) {
                println!("Error: Label '{}' is already used for this contact. Please choose a different label.", input);
            } else {
                break Some(input);
            }
        }
    } else {
        if existing_labels.contains(&"default".to_string()) {
            println!("Error: The 'default' label is already used for this contact.");
            // If default is taken, we must ask for a label or generate a unique one.
            // For simplicity and following user request, let's force a label if default is taken.
            loop {
                let input: String = Input::new()
                    .with_prompt("Default label is taken. Please enter a different Phone Label")
                    .interact_text()
                    .unwrap();
                
                if existing_labels.contains(&input) {
                    println!("Error: Label '{}' is already used. Please choose a different label.", input);
                } else {
                    break Some(input);
                }
            }
        } else {
            Some("default".to_string()) // Replace null with "default"
        }
    };

    let country_code: u16 = Input::new()
        .with_prompt("Country Code")
        .interact_text()
        .unwrap();

    let number: u32 = Input::new()
        .with_prompt("Phone Number")
        .interact_text()
        .unwrap();

    PhoneNumber {
        label,
        country_code,
        number,
    }
}

pub fn add_phone_to_contact(
    contact: &mut crate::contact::Contact,
    label: Option<String>,
    indicator: Option<u16>,
    number: Option<u32>,
) -> bool {
    let existing_labels: Vec<String> = contact
        .phones
        .as_ref()
        .map(|v| v.iter().filter_map(|p| p.label.clone()).collect())
        .unwrap_or_default();

    let new_phone = if let (Some(indicator), Some(number)) = (indicator, number) {
        let label_str = label.unwrap_or_else(|| "default".to_string());
        if existing_labels.contains(&label_str) {
            println!("Error: Telephone label '{}' is already used for this contact.", label_str);
            return false;
        }
        PhoneNumber {
            label: Some(label_str),
            country_code: indicator,
            number,
        }
    } else {
        create_phone_interactive(&existing_labels)
    };

    if let Some(ref mut phone_vec) = contact.phones {
        phone_vec.push(new_phone);
    } else {
        contact.phones = Some(vec![new_phone]);
    }
    true
}

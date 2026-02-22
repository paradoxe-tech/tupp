use dialoguer::{Input, Confirm};
use crate::models::Email;

pub fn create_email_interactive(existing_labels: &[String]) -> Email {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this email?")
        .default(false)
        .interact()
        .unwrap()
    {
        loop {
            let input: String = Input::new()
                .with_prompt("Email Label (e.g., Work, Personal)")
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
            loop {
                let input: String = Input::new()
                    .with_prompt("Default label is taken. Please enter a different Email Label")
                    .interact_text()
                    .unwrap();
                
                if existing_labels.contains(&input) {
                    println!("Error: Label '{}' is already used. Please choose a different label.", input);
                } else {
                    break Some(input);
                }
            }
        } else {
            Some("default".to_string())
        }
    };

    let address = Input::new()
        .with_prompt("Email Address")
        .interact_text()
        .unwrap();

    Email {
        label,
        address: Some(address),
    }
}

pub fn add_email_to_contact(
    contact: &mut crate::contact::Contact,
    label: Option<String>,
    address: Option<String>,
) -> bool {
    let existing_labels: Vec<String> = contact
        .emails
        .as_ref()
        .map(|v| v.iter().filter_map(|e| e.label.clone()).collect())
        .unwrap_or_default();

    let new_email = if let Some(address) = address {
        let label_str = label.unwrap_or_else(|| "default".to_string());
        if existing_labels.contains(&label_str) {
            println!("Error: Email label '{}' is already used for this contact.", label_str);
            return false;
        }
        Email {
            label: Some(label_str),
            address: Some(address),
        }
    } else {
        create_email_interactive(&existing_labels)
    };

    if let Some(ref mut email_vec) = contact.emails {
        email_vec.push(new_email);
    } else {
        contact.emails = Some(vec![new_email]);
    }
    true
}

use dialoguer::{Input, Confirm};
use crate::models::Email;

pub fn create_email_interactive() -> Email {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this email?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Email Label (e.g., Work, Personal)")
                .interact_text()
                .unwrap(),
        )
    } else {
        Some("default".to_string())
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

pub fn add_email_to_contact(contact: &mut crate::contact::Contact) {
    let new_email = create_email_interactive();

    if let Some(ref mut email_vec) = contact.email {
        email_vec.push(new_email);
    } else {
        contact.email = Some(vec![new_email]);
    }
}

use dialoguer::{Input, Confirm};
use crate::models::PhoneNumber;

pub fn create_phone_interactive() -> PhoneNumber {
    let label = if Confirm::new()
        .with_prompt("Do you want to enter a label for this phone number?")
        .default(false)
        .interact()
        .unwrap()
    {
        Some(
            Input::new()
                .with_prompt("Phone Label (e.g., Work, Mobile, Home)")
                .interact_text()
                .unwrap(),
        )
    } else {
        Some("default".to_string()) // Replace null with "default"
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

pub fn add_phone_to_contact(contact: &mut crate::contact::Contact) {
    let new_phone = create_phone_interactive();

    if let Some(ref mut phone_vec) = contact.phone {
        phone_vec.push(new_phone);
    } else {
        contact.phone = Some(vec![new_phone]);
    }
}

use dialoguer::{Confirm, Input};

pub fn add_birth_to_contact(contact: &mut crate::contact::Contact) {
    if Confirm::new()
        .with_prompt("Do you want to add a birth date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.birth_date = Some(crate::interactions::date::create_date_interactive());
    }

    if Confirm::new()
        .with_prompt("Do you want to add a birth location?")
        .default(false)
        .interact()
        .unwrap()
    {
        contact.identity.birth_location = Some(crate::interactions::address::create_address_interactive());
    }
    // Offer to edit birth names (initialized to identity values by default)
    if Confirm::new()
        .with_prompt("Do you want to edit birth first name?")
        .default(false)
        .interact()
        .unwrap()
    {
        let default_first = contact.identity.first_name.as_deref().unwrap_or_default().to_string();
        let first: String = Input::new()
            .with_prompt("Birth first name")
            .with_initial_text(default_first)
            .interact_text()
            .unwrap();

        contact.identity.birth_first_name = Some(first);
    }

    if Confirm::new()
        .with_prompt("Do you want to edit birth middle name?")
        .default(false)
        .interact()
        .unwrap()
    {
        let default_mid = contact.identity.middle_name.as_deref().unwrap_or_default().to_string();
        let mid: String = Input::new()
            .with_prompt("Birth middle name")
            .with_initial_text(default_mid)
            .interact_text()
            .unwrap();

        if mid.trim().is_empty() {
            contact.identity.birth_middle_name = None;
        } else {
            contact.identity.birth_middle_name = Some(mid);
        }
    }

    if Confirm::new()
        .with_prompt("Do you want to edit birth last name?")
        .default(false)
        .interact()
        .unwrap()
    {
        let default_last = contact.identity.last_name.as_deref().unwrap_or_default().to_string();
        let last: String = Input::new()
            .with_prompt("Birth last name")
            .with_initial_text(default_last)
            .interact_text()
            .unwrap();

        contact.identity.birth_last_name = Some(last);
    }
}

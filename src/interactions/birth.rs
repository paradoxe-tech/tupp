use dialoguer::{Confirm, Input};

pub fn add_birth_to_contact(
    contact: &mut crate::contact::Contact,
    birth_first_name: Option<String>,
    birth_middle_name: Option<String>,
    birth_last_name: Option<String>,
    day: Option<u8>,
    month: Option<u8>,
    year: Option<i32>,
) {
    if birth_first_name.is_some() || birth_middle_name.is_some() || birth_last_name.is_some() || day.is_some() || month.is_some() || year.is_some() {
        if let Some(first) = birth_first_name {
            contact.identity.birth_first_name = Some(first);
        }
        if let Some(middle) = birth_middle_name {
            contact.identity.birth_middle_name = Some(middle);
        }
        if let Some(last) = birth_last_name {
            contact.identity.birth_last_name = Some(last);
        }
        if day.is_some() || month.is_some() || year.is_some() {
            contact.identity.birth_date = Some(crate::interactions::date::create_date(year, month, day, None, None, None));
        }
        return;
    }

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

use dialoguer::Confirm;

pub fn add_death_to_contact(
    contact: &mut crate::contact::Contact,
    day: Option<u8>,
    month: Option<u8>,
    year: Option<i32>,
) {
    // Mark as deceased
    contact.identity.is_alive = false;

    if day.is_some() || month.is_some() || year.is_some() {
        contact.identity.death_date = Some(crate::interactions::date::create_date(year, month, day, None, None, None));
        return;
    }

    if Confirm::new()
        .with_prompt("Do you want to add a death date?")
        .default(true)
        .interact()
        .unwrap()
    {
        contact.identity.death_date = Some(crate::interactions::date::create_date_interactive());
    }

    if Confirm::new()
        .with_prompt("Do you want to add a death location?")
        .default(false)
        .interact()
        .unwrap()
    {
        contact.identity.death_location = Some(crate::interactions::address::create_address_interactive());
    }
}

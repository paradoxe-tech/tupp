use dialoguer::Input;

pub fn create_nickname_interactive() -> String {
    Input::new()
        .with_prompt("Nickname")
        .interact_text()
        .unwrap()
}

pub fn add_nickname_to_contact(contact: &mut crate::contact::Contact, nickname: Option<String>) -> bool {
    let new_nickname = nickname.unwrap_or_else(create_nickname_interactive);

    let already_exists = contact
        .nicknames
        .as_ref()
        .map(|v| v.contains(&new_nickname))
        .unwrap_or(false);
    if already_exists {
        println!("Error: Nickname '{}' is already set for this contact.", new_nickname);
        return false;
    }

    if let Some(ref mut nicknames) = contact.nicknames {
        nicknames.push(new_nickname);
    } else {
        contact.nicknames = Some(vec![new_nickname]);
    }
    true
}

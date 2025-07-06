use dialoguer::{Input, Confirm};
use crate::models::Social;

pub fn create_social_interactive() -> Social {
    let network = Input::new()
        .with_prompt("Social Network (e.g., Twitter, LinkedIn, Instagram)")
        .interact_text()
        .unwrap();

    let username = Input::new()
        .with_prompt("Username/Handle")
        .interact_text()
        .unwrap();

    Social {
        network,
        username: Some(username),
    }
}

pub fn add_social_to_contact(contact: &mut crate::contact::Contact) {
    let new_social = create_social_interactive();

    if let Some(ref mut social_vec) = contact.social {
        social_vec.push(new_social);
    } else {
        contact.social = Some(vec![new_social]);
    }
}

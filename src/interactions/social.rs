use dialoguer::Input;
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

pub fn add_social_to_contact(
    contact: &mut crate::contact::Contact,
    network: Option<String>,
    username: Option<String>,
) {
    let new_social = if let (Some(network), Some(username)) = (network, username) {
        Social {
            network,
            username: Some(username),
        }
    } else {
        create_social_interactive()
    };

    if let Some(ref mut social_vec) = contact.socials {
        social_vec.push(new_social);
    } else {
        contact.socials = Some(vec![new_social]);
    }
}

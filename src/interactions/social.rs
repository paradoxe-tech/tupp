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

    let label = Input::new()
        .with_prompt("Label (e.g., Personal, Work)")
        .default("default".to_string())
        .interact_text()
        .unwrap();

    Social {
        label: Some(label),
        network,
        username: Some(username),
    }
}

pub fn add_social_to_contact(
    contact: &mut crate::contact::Contact,
    label: Option<String>,
    network: Option<String>,
    username: Option<String>,
) -> bool {
    let new_social = if let (Some(network), Some(username)) = (network, username) {
        Social {
            label: Some(label.unwrap_or("default".to_string())),
            network,
            username: Some(username),
        }
    } else {
        create_social_interactive()
    };

    if let Some(ref mut socials) = contact.socials {
         socials.push(new_social);
    } else {
         contact.socials = Some(vec![new_social]);
    }
    true
}

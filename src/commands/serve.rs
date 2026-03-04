use std::env;
use std::path::PathBuf;

use tiny_http::{Header, Method, Response, Server};
use uuid::Uuid;

use crate::contact::{Contact, Link};
use crate::error::TuppError;
use crate::storage::{load_data, save_data};

const TOKEN_ENV: &str = "TUPP_API_TOKEN";

fn json_resp(body: String, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", "application/json").unwrap())
}

pub fn handle_serve_command(port: u16, file_path: &PathBuf) -> Result<(), TuppError> {
    let token = env::var(TOKEN_ENV).map_err(|_| {
        TuppError::Other(format!(
            "Environment variable {} is not set",
            TOKEN_ENV
        ))
    })?;

    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| TuppError::Other(format!("Failed to start server: {}", e)))?;

    eprintln!("tupp listening on http://{}", addr);

    for mut request in server.incoming_requests() {
        // --- Bearer auth ---
        let authorized = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Authorization"))
            .map(|h| h.value.as_str() == format!("Bearer {}", token))
            .unwrap_or(false);

        if !authorized {
            let _ = request.respond(json_resp(
                serde_json::json!({"error": "Unauthorized"}).to_string(),
                401,
            ));
            continue;
        }

        // Resolve route before consuming request for body reading
        #[derive(PartialEq)]
        enum Route {
            GetContacts,
            PostContacts,
            NotFound,
        }

        let route = match (
            request.method(),
            request.url().split('?').next().unwrap_or("").trim_end_matches('/'),
        ) {
            (Method::Get, "/contacts") => Route::GetContacts,
            (Method::Post, "/contacts") => Route::PostContacts,
            _ => Route::NotFound,
        };

        match route {
            // GET /contacts → return full data as JSON
            Route::GetContacts => {
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(data) => match serde_json::to_string(&data) {
                        Ok(json) => json_resp(json, 200),
                        Err(e) => json_resp(
                            serde_json::json!({"error": e.to_string()}).to_string(),
                            500,
                        ),
                    },
                };
                let _ = request.respond(resp);
            }

            // POST /contacts → create or update a contact
            Route::PostContacts => {
                // Read body
                let mut body = String::new();
                if let Err(e) = request.as_reader().read_to_string(&mut body) {
                    let _ = request.respond(json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        400,
                    ));
                    continue;
                }

                // Parse JSON
                let mut value: serde_json::Value = match serde_json::from_str(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = request.respond(json_resp(
                            serde_json::json!({"error": format!("Invalid JSON: {}", e)})
                                .to_string(),
                            400,
                        ));
                        continue;
                    }
                };

                // If no identifier → generate one (create mode)
                let is_update = value.get("identifier").is_some();
                if !is_update {
                    value["identifier"] = serde_json::json!(Uuid::new_v4().to_string());
                }

                // Deserialize into Contact (validates required fields)
                let contact: Contact = match serde_json::from_value(value) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = request.respond(json_resp(
                            serde_json::json!({"error": format!("Invalid contact: {}", e)})
                                .to_string(),
                            400,
                        ));
                        continue;
                    }
                };

                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        if is_update {
                            // Update existing contact
                            match data
                                .contacts
                                .iter()
                                .position(|c| c.identifier == contact.identifier)
                            {
                                Some(pos) => {
                                    data.contacts[pos] = contact;
                                    // Collect links before mutable iteration
                                    let contact_id = data.contacts[pos].identifier;
                                    let links_to_mirror: Vec<(Uuid, _)> = data.contacts[pos]
                                        .links
                                        .as_ref()
                                        .map(|ls| ls.iter().map(|l| (l.target, Contact::get_reciprocal_relation(&l.relation))).collect())
                                        .unwrap_or_default();
                                    // Add symmetric links to target contacts
                                    for (target_id, reciprocal) in links_to_mirror {
                                        if target_id == contact_id { continue; }
                                        if let Some(target) = data.contacts.iter_mut().find(|c| c.identifier == target_id) {
                                            let already_exists = target.links.as_ref()
                                                .map(|ls| ls.iter().any(|l| l.target == contact_id))
                                                .unwrap_or(false);
                                            if !already_exists {
                                                let new_link = Link { target: contact_id, relation: reciprocal };
                                                target.links.get_or_insert_with(Vec::new).push(new_link);
                                            }
                                        }
                                    }
                                    match save_data(file_path, &data) {
                                        Ok(_) => json_resp(
                                            serde_json::json!({"status": "updated"}).to_string(),
                                            200,
                                        ),
                                        Err(e) => json_resp(
                                            serde_json::json!({"error": e.to_string()})
                                                .to_string(),
                                            500,
                                        ),
                                    }
                                }
                                None => json_resp(
                                    serde_json::json!({"error": "Contact not found"}).to_string(),
                                    404,
                                ),
                            }
                        } else {
                            // Create new contact
                            let id = contact.identifier.to_string();
                            let contact_id = contact.identifier;
                            // Collect links before inserting to avoid borrow issues
                            let links_to_mirror: Vec<(Uuid, _)> = contact
                                .links
                                .as_ref()
                                .map(|ls| ls.iter().map(|l| (l.target, Contact::get_reciprocal_relation(&l.relation))).collect())
                                .unwrap_or_default();
                            data.contacts.push(contact);
                            // Add symmetric links to target contacts
                            for (target_id, reciprocal) in links_to_mirror {
                                if let Some(target) = data.contacts.iter_mut().find(|c| c.identifier == target_id) {
                                    let already_exists = target.links.as_ref()
                                        .map(|ls| ls.iter().any(|l| l.target == contact_id))
                                        .unwrap_or(false);
                                    if !already_exists {
                                        let new_link = Link { target: contact_id, relation: reciprocal };
                                        target.links.get_or_insert_with(Vec::new).push(new_link);
                                    }
                                }
                            }
                            match save_data(file_path, &data) {
                                Ok(_) => json_resp(serde_json::json!(id).to_string(), 201),
                                Err(e) => json_resp(
                                    serde_json::json!({"error": e.to_string()}).to_string(),
                                    500,
                                ),
                            }
                        }
                    }
                };

                let _ = request.respond(resp);
            }

            Route::NotFound => {
                let _ = request.respond(json_resp(
                    serde_json::json!({"error": "Not found"}).to_string(),
                    404,
                ));
            }
        }
    }

    Ok(())
}

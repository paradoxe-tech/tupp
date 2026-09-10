use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use tiny_http::{Header, Method, Request, Response, Server};
use uuid::Uuid;

use crate::contact::{Contact, Link};
use crate::error::TuppError;
use crate::group::Group;
use crate::institution::Institution;
use crate::storage::{load_data, save_data};

const TOKEN_ENV: &str = "TUPP_API_TOKEN";
const WORKER_THREADS: usize = 8;

fn cors_headers() -> Vec<Header> {
    vec![
        Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
        Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS").unwrap(),
        Header::from_bytes("Access-Control-Allow-Headers", "Authorization, Content-Type").unwrap(),
    ]
}

fn json_resp(body: String, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut resp = Response::from_string(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
    for h in cors_headers() {
        resp.add_header(h);
    }
    resp
}

fn binary_resp(body: Vec<u8>, content_type: &str, status: u16) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut resp = Response::from_data(body)
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", content_type).unwrap());
    for h in cors_headers() {
        resp.add_header(h);
    }
    resp
}

pub fn handle_serve_command(port: u16, file_path: &PathBuf) -> Result<(), TuppError> {
    let token = env::var(TOKEN_ENV).map_err(|_| {
        TuppError::Other(format!(
            "Environment variable {} is not set",
            TOKEN_ENV
        ))
    })?;

    let addr = format!("0.0.0.0:{}", port);
    let server = Arc::new(
        Server::http(&addr).map_err(|e| TuppError::Other(format!("Failed to start server: {}", e)))?,
    );

    eprintln!("tupp listening on http://{}", addr);

    // Guards access to the data file so concurrent requests can't corrupt it
    // with interleaved read-modify-write cycles, while still letting workers
    // handle slow clients (accepting/reading/writing the response) in parallel.
    let data_lock = Arc::new(Mutex::new(()));

    let mut workers = Vec::with_capacity(WORKER_THREADS);
    for _ in 0..WORKER_THREADS {
        let server = Arc::clone(&server);
        let data_lock = Arc::clone(&data_lock);
        let file_path = file_path.clone();
        let token = token.clone();
        workers.push(thread::spawn(move || loop {
            match server.recv() {
                Ok(request) => handle_request(request, &file_path, &token, &data_lock),
                Err(e) => {
                    eprintln!("tupp: error receiving request: {}", e);
                    break;
                }
            }
        }));
    }

    for worker in workers {
        let _ = worker.join();
    }

    Ok(())
}

fn handle_request(mut request: Request, file_path: &PathBuf, token: &str, data_lock: &Mutex<()>) {
    // --- Bearer auth ---
    let authorized = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Authorization"))
        .map(|h| h.value.as_str() == format!("Bearer {}", token))
        .unwrap_or(false);

    // Preflight CORS — no auth required
    if request.method() == &Method::Options {
        let mut resp = Response::empty(204);
        for h in cors_headers() {
            resp.add_header(h);
        }
        let _ = request.respond(resp);
        return;
    }

    if !authorized {
        let _ = request.respond(json_resp(
            serde_json::json!({"error": "Unauthorized"}).to_string(),
            401,
        ));
        return;
    }

    // Resolve route before consuming request for body reading
        enum Route {
            GetContacts,
            PostContacts,
            DeleteContact(Uuid),
            GetContactPhoto(Uuid),
            GetGroups,
            PostGroups,
            DeleteGroup(Uuid),
            GetInstitutions,
            PostInstitutions,
            DeleteInstitution(Uuid),
            NotFound,
        }

        let path = request.url().split('?').next().unwrap_or("").trim_end_matches('/').to_string();
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        let route = match (request.method(), segments.as_slice()) {
            (Method::Get, ["contacts"]) => Route::GetContacts,
            (Method::Post, ["contacts"]) => Route::PostContacts,
            (Method::Delete, ["contacts", id]) => match Uuid::parse_str(id) {
                Ok(uuid) => Route::DeleteContact(uuid),
                Err(_) => Route::NotFound,
            },
            (Method::Get, ["contacts", id, "photo"]) => match Uuid::parse_str(id) {
                Ok(uuid) => Route::GetContactPhoto(uuid),
                Err(_) => Route::NotFound,
            },
            (Method::Get, ["groups"]) => Route::GetGroups,
            (Method::Post, ["groups"]) => Route::PostGroups,
            (Method::Delete, ["groups", id]) => match Uuid::parse_str(id) {
                Ok(uuid) => Route::DeleteGroup(uuid),
                Err(_) => Route::NotFound,
            },
            (Method::Get, ["institutions"]) => Route::GetInstitutions,
            (Method::Post, ["institutions"]) => Route::PostInstitutions,
            (Method::Delete, ["institutions", id]) => match Uuid::parse_str(id) {
                Ok(uuid) => Route::DeleteInstitution(uuid),
                Err(_) => Route::NotFound,
            },
            _ => Route::NotFound,
        };

        match route {
            // GET /contacts → return full data as JSON
            Route::GetContacts => {
                let _guard = data_lock.lock().unwrap();
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
                    return;
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
                        return;
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
                        return;
                    }
                };

                let _guard = data_lock.lock().unwrap();
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

            Route::DeleteContact(id) => {
                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        let initial_len = data.contacts.len();
                        data.contacts.retain(|c| c.identifier != id);
                        if data.contacts.len() == initial_len {
                            json_resp(
                                serde_json::json!({"error": "Contact not found"}).to_string(),
                                404,
                            )
                        } else {
                            match save_data(file_path, &data) {
                                Ok(_) => json_resp(
                                    serde_json::json!({"status": "deleted"}).to_string(),
                                    200,
                                ),
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

            // GET /contacts/{id}/photo → serve the downloaded photo, if any
            Route::GetContactPhoto(id) => {
                let resp = match crate::storage::find_photo(&id) {
                    Some((bytes, content_type)) => binary_resp(bytes, content_type, 200),
                    None => json_resp(
                        serde_json::json!({"error": "No photo for this contact"}).to_string(),
                        404,
                    ),
                };
                let _ = request.respond(resp);
            }

            Route::GetGroups => {
                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(data) => match serde_json::to_string(&data.groups) {
                        Ok(json) => json_resp(json, 200),
                        Err(e) => json_resp(
                            serde_json::json!({"error": e.to_string()}).to_string(),
                            500,
                        ),
                    },
                };
                let _ = request.respond(resp);
            }

            Route::PostGroups => {
                let mut body = String::new();
                if let Err(e) = request.as_reader().read_to_string(&mut body) {
                    let _ = request.respond(json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        400,
                    ));
                    return;
                }

                #[derive(serde::Deserialize)]
                struct NewGroup {
                    name: String,
                    parent: Option<Uuid>,
                }

                let new_group_req: NewGroup = match serde_json::from_str(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = request.respond(json_resp(
                            serde_json::json!({"error": format!("Invalid group: {}", e)})
                                .to_string(),
                            400,
                        ));
                        return;
                    }
                };

                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        let new_group = Group::new(new_group_req.name);
                        let id = new_group.identifier;
                        let added = match new_group_req.parent {
                            Some(parent_id) => Group::find_parent_and_add_recursive(
                                &mut data.groups,
                                &parent_id,
                                new_group,
                            ),
                            None => {
                                data.groups.push(new_group);
                                true
                            }
                        };

                        if !added {
                            json_resp(
                                serde_json::json!({"error": "Parent group not found"})
                                    .to_string(),
                                404,
                            )
                        } else {
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

            Route::DeleteGroup(id) => {
                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        if Group::delete_group_recursive(&mut data.groups, &id) {
                            for contact in &mut data.contacts {
                                if let Some(ref mut groups) = contact.groups {
                                    groups.remove(&id);
                                }
                            }
                            match save_data(file_path, &data) {
                                Ok(_) => json_resp(
                                    serde_json::json!({"status": "deleted"}).to_string(),
                                    200,
                                ),
                                Err(e) => json_resp(
                                    serde_json::json!({"error": e.to_string()}).to_string(),
                                    500,
                                ),
                            }
                        } else {
                            json_resp(
                                serde_json::json!({"error": "Group not found"}).to_string(),
                                404,
                            )
                        }
                    }
                };
                let _ = request.respond(resp);
            }

            Route::GetInstitutions => {
                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(data) => match serde_json::to_string(&data.institutions) {
                        Ok(json) => json_resp(json, 200),
                        Err(e) => json_resp(
                            serde_json::json!({"error": e.to_string()}).to_string(),
                            500,
                        ),
                    },
                };
                let _ = request.respond(resp);
            }

            Route::PostInstitutions => {
                let mut body = String::new();
                if let Err(e) = request.as_reader().read_to_string(&mut body) {
                    let _ = request.respond(json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        400,
                    ));
                    return;
                }

                #[derive(serde::Deserialize)]
                struct NewInstitution {
                    name: String,
                    parent: Option<Uuid>,
                }

                let new_institution_req: NewInstitution = match serde_json::from_str(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = request.respond(json_resp(
                            serde_json::json!({"error": format!("Invalid institution: {}", e)})
                                .to_string(),
                            400,
                        ));
                        return;
                    }
                };

                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        let new_institution = Institution::new(new_institution_req.name);
                        let id = new_institution.identifier;
                        let added = match new_institution_req.parent {
                            Some(parent_id) => Institution::find_parent_and_add_recursive(
                                &mut data.institutions,
                                &parent_id,
                                new_institution,
                            ),
                            None => {
                                data.institutions.push(new_institution);
                                true
                            }
                        };

                        if !added {
                            json_resp(
                                serde_json::json!({"error": "Parent institution not found"})
                                    .to_string(),
                                404,
                            )
                        } else {
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

            Route::DeleteInstitution(id) => {
                let _guard = data_lock.lock().unwrap();
                let resp = match load_data(file_path) {
                    Err(e) => json_resp(
                        serde_json::json!({"error": e.to_string()}).to_string(),
                        500,
                    ),
                    Ok(mut data) => {
                        let mut removed_ids = Vec::new();
                        if let Some(institution) = Institution::find_institution_by_id_recursive(&data.institutions, &id) {
                            Institution::collect_ids_recursive(std::slice::from_ref(institution), &mut removed_ids);
                        }

                        if !Institution::delete_institution_recursive(&mut data.institutions, &id) {
                            json_resp(
                                serde_json::json!({"error": "Institution not found"}).to_string(),
                                404,
                            )
                        } else {
                            for contact in &mut data.contacts {
                                if let Some(ref mut positions) = contact.positions {
                                    positions.retain(|p| !removed_ids.contains(&p.institution));
                                }
                            }
                            match save_data(file_path, &data) {
                                Ok(_) => json_resp(
                                    serde_json::json!({"status": "deleted"}).to_string(),
                                    200,
                                ),
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

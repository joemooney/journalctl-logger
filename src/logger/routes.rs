use std::borrow::BorrowMut;

pub use super::state::LoggerState;
use super::Db;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StartLoggingRequest {
    log_path: String,
    component: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StopLoggingRequest {
    component: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggingResponse {
    components: Vec<ComponentLoggingResponse>,
    request_status: bool,
    request_message: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComponentLoggingResponse {
    log_path: Option<String>,
    previous_log_path: Option<String>,
    component: String,
    active: bool,
}

// PUT is idempotent, repeated calls return same value
// whereas POST is not idempotent

#[openapi]
#[post("/start", format = "json", data = "<req>")]
pub fn start(req: Json<StartLoggingRequest>, db: State<Db>) -> Json<LoggingResponse> {
    let mut db = db.lock().unwrap();
    db.call_count += 1;
    let mut resp = LoggingResponse::default();
    resp.request_status = true;
    resp.components = vec![];
    match req.component {
        Some(component_name) => start_component(component_name, req.log_path, &mut resp, &mut db),
        None => (),
    }
    Json(resp)
}

#[openapi]
#[post("/stop", format = "json", data = "<req>")]
pub fn stop(req: Json<StopLoggingRequest>, db: State<Db>) -> Json<LoggingResponse> {
    let mut db = db.lock().unwrap();
    db.call_count += 1;
    let mut resp = LoggingResponse::default();
    resp.request_status = true;
    resp.components = vec![];
    match &req.component {
        Some(component_name) => stop_component(component_name, &mut resp, &mut db),
        None => (),
    }
    Json(resp)
}

fn start_component(
    component_name: String,
    log_path: String,
    resp: &mut LoggingResponse,
    db: &mut LoggerState,
) {
    if !db.components.contains_key(&component_name) {
        resp.request_status = false;
        resp.request_message = Some("Invalid component".to_string());
    } else {
        let &mut component = db.components[&component_name].borrow_mut();
        if Some(log_path.clone()) == component.log_path && db.active {
            resp.request_status = false;
            resp.request_message = Some("Already logging to this log_path".to_string());
        } else {
            if !component.log_path.is_none() {
                component.previous_log_path = component.log_path.clone();
            }
            component.log_path = Some(log_path.clone());
            component.active = true;
            super::run_command(&mut component);
            let mut comp_resp = ComponentLoggingResponse::default();
            comp_resp.component = component_name.clone();
            comp_resp.log_path = component.log_path.clone();
            comp_resp.previous_log_path = component.previous_log_path.clone();
            comp_resp.active = component.active;
            resp.components.push(comp_resp);
            resp.request_status = true;
            resp.request_message = Some("Logging started".to_string());
        }
    }
}

fn stop_component(component_name: &String, resp: &mut LoggingResponse, db: &mut LoggerState) {
    if !db.components.contains_key(component_name) {
        resp.request_status = false;
        resp.request_message = Some("Invalid component".to_string());
    } else {
        let component = db.components[component_name].borrow_mut();
        if !component.active {
            resp.request_status = false;
            resp.request_message = Some("No logging was active".to_string());
        } else {
            match &mut component.command {
                Some(command) => match command.kill() {
                    Ok(()) => println!("Stopped logging command"),
                    Err(err) => eprintln!("Failed to stop logging command: {:?}", err),
                },
                None => eprintln!("No command to stop!"),
            }
            component.previous_log_path = component.log_path.clone();
            component.log_path = None;
            component.active = false;
            let mut comp_resp = ComponentLoggingResponse::default();
            comp_resp.log_path = component.previous_log_path.clone();
            comp_resp.previous_log_path = component.previous_log_path.clone();
            comp_resp.active = component.active;
            comp_resp.component = component_name.clone();
            resp.components.push(comp_resp);
            resp.request_status = true;
            resp.request_message = Some("Logging stopped".to_string());
        }
    }
}

#[openapi]
#[get("/status", format = "json")]
pub fn status(db: State<Db>) -> Json<LoggingResponse> {
    let mut db = db.lock().unwrap();
    let message = if db.active {
        Some("Logging active".to_string())
    } else {
        Some("No logging active".to_string())
    };
    db.call_count += 1;
    let mut resp = LoggingResponse::default();
    resp.request_status = true;
    resp.components = vec![];
    for component in db.components.values() {
        resp.components.push(ComponentLoggingResponse {
            component: component.name.clone(),
            log_path: component.log_path.clone(),
            previous_log_path: component.previous_log_path.clone(),
            active: component.active,
        })
    }
    Json(resp)
}

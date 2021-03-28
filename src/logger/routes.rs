use super::Db;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoggingResponse {
    path: Option<String>,
    previous_path: Option<String>,
    active: bool,
    request_status: bool,
    request_message: Option<String>,
}

// PUT is idempotent, repeated calls return same value
// whereas POST is not idempotent

#[openapi]
#[post("/start", format = "json", data = "<req>")]
pub fn start(req: Json<StartRequest>, db: State<Db>) -> Json<LoggingResponse> {
    let mut db = db.lock().unwrap();
    let mut status = true;
    let mut message = Some("Logging started".to_string());
    db.call_count += 1;
    if Some(req.path.clone()) == db.path && db.active {
        status = false;
        message = Some("Already logging to this path".to_string());
    } else {
        db.active = true;
        if !db.path.is_none() {
            db.previous_path = db.path.clone();
        }
        db.path = Some(req.path.clone());
        super::run_command(&mut db);
    }
    Json(LoggingResponse {
        path: db.path.clone(),
        previous_path: db.previous_path.clone(),
        active: db.active,
        request_status: status,
        request_message: message,
    })
}

#[openapi]
#[post("/stop")]
pub fn stop(db: State<Db>) -> Json<LoggingResponse> {
    let mut db = db.lock().unwrap();
    let mut status = true;
    let mut message = Some("Logging stopped".to_string());
    db.call_count += 1;
    if !db.active {
        status = false;
        message = Some("No logging was active".to_string());
    } else {
        match &mut db.command {
            Some(command) => match command.kill() {
                Ok(()) => println!("Stopped logging command"),
                Err(err) => eprintln!("Failed to stop logging command: {:?}", err),
            },
            None => eprintln!("No command to stop!"),
        }
        db.active = true;
        db.previous_path = db.path.clone();
        db.path = None;
        db.active = false;
    }
    Json(LoggingResponse {
        path: db.path.clone(),
        previous_path: db.previous_path.clone(),
        active: db.active,
        request_status: status,
        request_message: message,
    })
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
    Json(LoggingResponse {
        path: db.path.clone(),
        previous_path: db.previous_path.clone(),
        active: db.active,
        request_status: true,
        request_message: message,
    })
}

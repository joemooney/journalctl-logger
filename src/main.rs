#![feature(proc_macro_hygiene, decl_macro)]

/*
This program is an example of a rocket http server that
uses rocket_okapi openapi crate to provide a FastAPI like
implemntation in Rust.

This also shows how to store State information that
persists and is thread safe.
 */
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_okapi;

use exitfailure::ExitFailure;
// use failure::ResultExt;
use rocket::Rocket;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::{fs::File, process::Stdio};
use structopt::StructOpt;

#[derive(Debug)]
struct LoggerState {
    id: u64,
    path: Option<String>,
    previous_path: Option<String>,
    call_count: u32,
    active: bool,
    command: Option<Child>,
}

impl LoggerState {
    fn new() -> LoggerState {
        LoggerState {
            id: 0,
            path: None,
            previous_path: None,
            command: None,
            call_count: 0,
            active: false,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
struct StartRequest {
    path: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct LoggingResponse {
    path: Option<String>,
    previous_path: Option<String>,
    active: bool,
    request_status: bool,
    request_message: Option<String>,
}

// PUT is idempotent, repeated calls return same value

#[openapi]
#[post("/start", format = "json", data = "<req>")]
fn start(req: Json<StartRequest>, db: State<Db>) -> Json<LoggingResponse> {
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
        run_command(&mut db);
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
fn stop(db: State<Db>) -> Json<LoggingResponse> {
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
fn status(db: State<Db>) -> Json<LoggingResponse> {
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

/// A simple in-memory DB to store logging state
type Db = Mutex<LoggerState>;

fn run_command(db: &mut LoggerState) {
    println!("Starting command");
    let out;
    match &db.path {
        Some(path) => {
            out = File::create(path).unwrap();
        }
        None => {
            eprintln!("No file path");
            return;
        }
    }
    // let args = shlex::split("/usr/bin/tail -f /tmp/foo.txt").unwrap();
    let args = shlex::split("/usr/bin/ssh localhost journalctl -k -f").unwrap();

    db.command = Some(
        Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::from(out))
            .spawn()
            .expect("failed to execute child"),
    );
}

/*
fn wait_command(db: &Db) {
    let ecode = db.child.wait()
                    .expect("failed to wait on child");

    assert!(ecode.success());
}
 */

fn build_app() -> Rocket {
    rocket::ignite()
        .manage(Mutex::new(LoggerState::new()))
        .mount("/", routes_with_openapi![status, start, stop])
        .mount(
            "/docs/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();
    if opt.status {
        let resp =
            reqwest::blocking::get("http://localhost:8000/status")?.json::<LoggingResponse>()?;
        println!("{:#?}", resp);
    } else {
        build_app().launch();
    }
    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag. The name of the
    // argument will be, by default, based on the name of the field.
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Ask server for status
    #[structopt(short, long)]
    status: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /*
    /// Set speed
    #[structopt(short, long, default_value = "42")]
    speed: f64,
    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
     */
    // the long option will be translated by default to kebab case,
    // i.e. `--nb-cars`.
    /// Number of cars
    #[structopt(short = "c", long)]
    nb_cars: Option<i32>,

    /// admin_level to consider
    #[structopt(short, long)]
    level: Vec<String>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::build_app;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;

    #[test]
    fn status() {
        let client = Client::new(build_app()).expect("Could not launch server");
        let req = client.get("/status").header(ContentType::JSON);
        let mut resp = req.dispatch();
        let r = resp.body_string();
        println!("{}", r.clone().unwrap());
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            r,
            Some(r#"{"path":null,"previousPath":null,"active":false,"requestStatus":true,"requestMessage":"No logging active"}"#.to_string())
        );
    }
    #[test]
    fn start() {
        let client = Client::new(build_app()).expect("Could not launch server");
        let req = client
            .post("/start")
            .header(ContentType::JSON)
            .body(r#"{"path": "/tmp/foo.txt"}"#);
        let mut resp = req.dispatch();
        let r = resp.body_string();
        println!("{}", r.clone().unwrap());
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            r,
            Some(r#"{"path":"/tmp/foo.txt","previousPath":null,"active":true,"requestStatus":true,"requestMessage":"Logging started"}"#.to_string())
        );
    }
}

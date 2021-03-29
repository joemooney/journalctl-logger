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
// use rocket::State;
// use rocket_contrib::json::Json;
// use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// use std::process::Command;
// use std::sync::Mutex;
// use std::{fs::File, process::Stdio};
use structopt::StructOpt;
mod http_server;
mod logger;
use logger::LoggingResponse;

/*
fn wait_command(db: &Db) {
    let ecode = db.child.wait()
                    .expect("failed to wait on child");

    assert!(ecode.success());
}
 */

fn main() -> Result<(), ExitFailure> {
    // let port = env::var("PORT").unwrap_or("8000".to_string());
    let opt = Opt::from_args();
    if opt.status {
        let resp =
            reqwest::blocking::get("http://localhost:8000/status")?.json::<LoggingResponse>()?;
        println!("{:#?}", resp);
    } else {
        http_server::build_app(opt).launch();
    }
    Ok(())
}

#[derive(StructOpt, Debug, PartialEq)]
#[structopt(name = "basic")]
pub struct Opt {
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

    /// Port for http server
    #[structopt(short, long, default_value = "38000")]
    port: u32,

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

// unit test code - store in same file as tested function
// compiles test code with cargo test, not cargo build
#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_server;
    use rocket::http::{ContentType, Status};
    use rocket::local::Client;

    #[test]
    fn status() {
        let no_opts: Vec<String> = vec![];
        let opts = Opt::from_iter(no_opts);
        let client = Client::new(http_server::build_app(opts)).expect("Could not launch server");
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
        let no_opts: Vec<String> = vec![];
        let opts = Opt::from_iter(no_opts);
        let client = Client::new(http_server::build_app(opts)).expect("Could not launch server");
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

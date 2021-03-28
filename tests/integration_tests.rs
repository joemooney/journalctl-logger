use assert_cmd::prelude::*;
use rocket::http::{ContentType, Status};
use rocket::local::Client;
use std::process::Command;

/**
 Integration tests are entirely external to your library.
 They use your library in the same way any other code would,
 which means they can only call functions that are part of your
 library’s public API.

 Their purpose is to test whether many parts of your library
 work together correctly. Units of code that work correctly
 on their own could have problems when integrated, so test
 coverage of the integrated code is important as well.

*/
/*
#[test]
fn run_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    Command::new("/home/jpm/rust/journalctl-logger/target/debug/journalclt-logger")
        .args(&["--status"])
        .assert()
        .success();
    Ok(())
}
 */

#[test]
fn run_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("journalctl-logger").expect("binary found");
    let assert = cmd.arg("--status").assert();
    assert.failure();
    Ok(())
}

#[test]
fn run_with_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("journalctl-logger").expect("binary found");
    let assert = cmd.arg("--help").assert();
    assert.success();
    Ok(())
}

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

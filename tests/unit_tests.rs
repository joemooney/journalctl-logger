#[cfg(test)]
use super::*;
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

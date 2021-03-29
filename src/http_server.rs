use crate::logger;
use crate::Opt;
use logger::LoggerState;
use rocket::Rocket;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use std::env;
use std::sync::Mutex;

pub fn build_app(opt: Opt) -> Rocket {
    env::set_var("ROCKET_PORT", opt.port.to_string());
    rocket::ignite()
        .manage(Mutex::new(LoggerState::new()))
        .mount(
            "/",
            routes_with_openapi![logger::status, logger::start, logger::stop],
        )
        .mount(
            "/docs/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
}

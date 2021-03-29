mod routes;
mod state;
pub use routes::LoggingResponse;
pub use routes::*;
pub use state::ComponentState;
pub use state::Db;
pub use state::LoggerState;
use std::process::Command;
use std::{fs::File, process::Stdio};

fn run_command(db: &mut ComponentState) {
    println!("Starting command");
    let out;
    match &db.log_path {
        Some(path) => {
            out = File::create(path).unwrap();
        }
        None => {
            eprintln!("No file path");
            return;
        }
    }
    let cmd = "/usr/bin/ssh localhost journalctl -k -f";
    // let args = shlex::split("/usr/bin/tail -f /tmp/foo.txt").unwrap();
    let args = shlex::split(cmd).unwrap();

    db.command = Some(
        Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::from(out))
            .spawn()
            .expect("failed to execute child"),
    );
}

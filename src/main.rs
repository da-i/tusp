mod job;
mod scheduler;
mod executor;
mod repository;
mod daemon;

use std::env;

const IPC_SOCKET_PATH: &str = "/tmp/tusp.sock";

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("daemon") => daemon::run_daemon(IPC_SOCKET_PATH),
        Some("submit") => {
            if args.len() < 3 {
                eprintln!("Usage: tusp submit <command>");
                std::process::exit(1);
            }

            let command = args[2..].join(" ");
            if let Err(error) = daemon::submit_job(IPC_SOCKET_PATH, &command) {
                eprintln!("Failed to submit job: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage:\n  tusp daemon\n  tusp submit <command>");
            std::process::exit(1);
        }
    }
}


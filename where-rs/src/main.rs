mod config;
mod servers;
mod ui;
mod args;

use clap::Parser;
use args::Args;
use whrd::error::WhereResult;
use config::{Config, Server};

fn main() {
    if let Err(e) = start_client() {
        eprintln!("where: {}", e);
        std::process::exit(1);
    }
}

fn start_client() -> WhereResult<()> {
    let args = Args::parse();
    let config = Config::build(args);
    let global_config = config.global;

    let servers: Vec<Server> = config.server;
    let mut sessions = vec![];

    for server in servers {
        let res = match server.process(&global_config) {
            Ok(collection) => {
                collection
            }
            Err(e) => {
                eprintln!("where: {e}");

                if !server.failsafe.unwrap_or(false) {
                    std::process::exit(1);
                }

                continue
            }
        };

        sessions.extend(res.into_vec());
    }

    ui::print_summary(sessions, global_config);
    Ok(())
}

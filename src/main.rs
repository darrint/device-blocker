#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate nickel;
extern crate serde_json;
extern crate serde;
extern crate chrono;
extern crate clap;

mod script;
mod types;
mod server;
mod schedule;
mod script_handler;
mod files;
mod config;
mod app_server;
mod errors {
    error_chain!{
        errors {
            RequestError(msg: String) {
                description("problem with request")
                display("problem in request: {}", msg)
            }
        }
    }
}

use schedule::World;
use std::sync::{Arc, Mutex};
use server::run_server;
use script_handler::ScriptHandler;
use clap::{Arg, App};
use files::read_json_file;
use config::{Config, reconcile_config};
use app_server::AppServer;

use errors::{Result, ResultExt};

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new("Device Route Manater")
        .version("0.1")
        .arg(Arg::with_name("config_file")
            .long("config-file")
            .short("c")
            .help("Config file")
            .value_name("FILE")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("debug")
            .long("debug")
            .short("d")
            .help("Prints scripts instead of running them"))
        .get_matches();

    let script_handler = if matches.is_present("debug") {
        ScriptHandler::PrintScript
    } else {
        ScriptHandler::RunScript
    };

    let config_file: &str = matches.value_of("config_file")
        .ok_or("Config file argument required")?;
    let config: Config = read_json_file(&config_file).chain_err(|| "Failed to read config file")?;
    let mut internal = AppServer {
        config_file: config_file.to_owned(),
        config: config.clone(),
        world: World::default(),
        handler: script_handler,
    };
    let mut devs = std::collections::BTreeSet::new();
    app_server::read_dhcp_devices(&config.dhcp_lease_file, &mut devs)
        .chain_err(|| "Failed to read dhcp leases file")?;
    internal.read_or_create_world()?;
    let reconcile_result = reconcile_config(&internal.config, &devs, &mut internal.world);
    if reconcile_result.updated_world {
        internal.write_world()?;
    }

    let sync_app_server = Arc::new(Mutex::new(internal));

    run_server(sync_app_server.clone());

    Ok(())
}

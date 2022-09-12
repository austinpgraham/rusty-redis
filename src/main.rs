#![cfg_attr(test, feature(proc_macro_hygiene))]

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate mocktopus;

use std::env;

use cli::cmd::Executable;
use structopt::StructOpt;

mod cli;
mod cluster;
mod local;

fn main() {
    // Set the default log level if not provided
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    pretty_env_logger::init();

    let root_args = cli::RootCommand::from_args();
    let cmd_result = match root_args.cmd {
        cli::ClusterCommand::Config(config_args) => match config_args {
            cli::ClusterConfig::Ls(ls_command) => ls_command.execute(),
        },
        cli::ClusterCommand::Cluster(cluster_args) => match cluster_args {
            cli::ClusterRuntime::Start(start_command) => start_command.execute(),
            cli::ClusterRuntime::Stop(stop_command) => stop_command.execute(),
        },
    };

    if let Err(result_error) = cmd_result {
        error!("{}", result_error);
    } else {
        info!("Command executed successfully.");
    }
}

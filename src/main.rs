extern crate pretty_env_logger;
#[macro_use] extern crate log;

use cli::cmd::Executable;
use structopt::StructOpt;

mod cli;

fn main() {
    pretty_env_logger::init();

    let root_args = cli::RootCommand::from_args();
    let cmd_result = match root_args.cmd {
        cli::ClusterCommand::Config(config_args) => match config_args {
            cli::ClusterConfig::Ls(ls_command) => ls_command.execute()
        }
    };

    if let Err(result_error) = cmd_result {
        error!("{}", result_error);
    } else {
        info!("Command executed successfully.");
    }
}

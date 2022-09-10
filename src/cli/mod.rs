use structopt::StructOpt;

pub mod ls;
pub mod cmd;

#[derive(Debug, StructOpt)]
pub enum ClusterConfig {
    Ls(ls::ClusterLs)
}

#[derive(Debug, StructOpt)]
pub enum ClusterCommand {
    Config(ClusterConfig)
}

#[derive(Debug, StructOpt)]
pub struct RootCommand {
    #[structopt(subcommand)]
    pub cmd: ClusterCommand
}

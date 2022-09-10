use structopt::StructOpt;

pub mod ls;
pub mod cmd;
pub mod start;

#[derive(Debug, StructOpt)]
pub enum ClusterConfig {
    Ls(ls::ClusterLs)
}

#[derive(Debug, StructOpt)]
pub enum ClusterRuntime {
    Start(start::ClusterStart)
}

#[derive(Debug, StructOpt)]
pub enum ClusterCommand {
    Config(ClusterConfig),
    Cluster(ClusterRuntime)
}

#[derive(Debug, StructOpt)]
pub struct RootCommand {
    #[structopt(subcommand)]
    pub cmd: ClusterCommand
}

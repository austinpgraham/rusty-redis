use structopt::StructOpt;

use crate::cluster::runtime::check_cluster_health;

use super::cmd::Executable;

#[derive(Debug, StructOpt)]
pub struct ClusterCheck {
    #[structopt(
        name = "cluster-host",
        short = "-h",
        long = "--cluster-host",
        default_value = "127.0.0.1"
    )]
    cluster_host: String,
}

impl Executable for ClusterCheck {
    fn execute(&self) -> Result<(), String> {
        check_cluster_health(&self.cluster_host)
    }
}

use structopt::StructOpt;

use crate::cluster::runtime::stop_cluster;

use super::cmd::Executable;

#[derive(Debug, StructOpt)]
pub struct ClusterStop {}

impl Executable for ClusterStop {
    fn execute(&self) -> Result<(), String> {
        stop_cluster()
    }
}

use std::path::PathBuf;

use structopt::StructOpt;

use crate::cluster::{
    config::{aggregate_config_files, resolve_base_file_path},
    runtime::start_cluster,
};

use super::cmd::Executable;

#[derive(Debug, StructOpt)]
pub struct ClusterStart {
    #[structopt(
        name = "base-dir",
        short = "-b",
        long = "--base-dir",
        parse(from_os_str)
    )]
    base_dir: Option<PathBuf>,

    #[structopt(
        name = "cluster-host",
        short = "-h",
        long = "--cluster-host",
        default_value = "127.0.0.1"
    )]
    cluster_host: String,
}

impl Executable for ClusterStart {
    fn execute(&self) -> Result<(), String> {
        let base_conf_path = resolve_base_file_path(&self.base_dir);

        match aggregate_config_files(&base_conf_path) {
            Ok(conf_list) => {
                if conf_list.len() <= 0 {
                    Err(format!(
                        "No configuration files found in path: {}",
                        base_conf_path.as_os_str().to_str().unwrap_or("DIR_ERROR")
                    ))
                } else {
                    start_cluster(&self.cluster_host, conf_list)
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

use std::path::PathBuf;

use structopt::StructOpt;

use crate::{
    cli::cmd::Executable,
    cluster::config::{list_conf_files, resolve_base_file_path},
};

#[derive(Debug, StructOpt)]
pub struct ClusterLs {
    #[structopt(
        name = "base-dir",
        short = "-b",
        long = "--base-dir",
        parse(from_os_str)
    )]
    base_dir: Option<PathBuf>,
}

impl Executable for ClusterLs {
    fn execute(&self) -> Result<(), String> {
        let base_path = resolve_base_file_path(&self.base_dir);

        if !base_path.exists() {
            Err("The given base_dir does not exist.".to_string())
        } else {
            list_conf_files(&base_path)
        }
    }
}

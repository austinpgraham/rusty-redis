use std::{path::PathBuf, str::FromStr};

use structopt::StructOpt;

use super::cmd::Executable;

#[derive(Debug, StructOpt)]
pub struct ClusterLs {
    #[structopt(
        name="base-dir",
        short = "-b",
        long = "--base-dir",
        parse(from_os_str)
    )]
    base_dir: Option<PathBuf>
}

impl Executable for ClusterLs {
    fn execute(&self) -> Result<(), String> {
        let base_path = match &self.base_dir {
            Some(path)=> path.clone(),
            None => PathBuf::from_str("/usr/local/etc/redis/cluster").unwrap()
        };

        if !base_path.exists() {
            Err("The given base_dir does not exist.".to_string())
        } else {
            Ok(())
        }
    }
}

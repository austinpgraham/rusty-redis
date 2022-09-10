use std::{
    path::PathBuf,
    process::Command
};

use crate::cluster::config::read_conf_file;

#[inline]
fn spawn_server_process(conf_file: String) -> Result<(), ()> {
    match Command::new("redis-server")
                                .arg(conf_file)
                                .spawn() {
                                    Ok(_) => Ok(()),
                                    Err(_) => Err(())
                                }
}

pub fn start_cluster(cluster_host: &String, conf_files: Vec<String>) -> Result<(), String> {
    let valid_files: Vec<PathBuf> = conf_files.iter()
                                .map(|path_str| PathBuf::from(path_str))
                                .filter(|path| path.exists())
                                .filter(|path| {
                                    {
                                        let file_path = path.to_str().unwrap().to_string(); 
                                        match spawn_server_process(file_path.clone()) {
                                            Ok(_) => {
                                                info!("Process with conf {} successfully started.", file_path);
                                                Ok(())
                                            },
                                            Err(_) => {
                                                error!("Process with conf {} failed to spawn.", file_path);
                                                Err(())
                                            }
                                        }
                                    }.is_ok()
                                })
                                .collect();
    
    if valid_files.len() <= 0 {
        Err("No valid configuration files were found.".to_string())
    } else {
        let _nodes: Vec<String> = valid_files.iter()
                                            .map(|file| read_conf_file(file))
                                            .filter(|file_result| file_result.is_ok())
                                            .map(|file_result| {
                                                let property_map = file_result.unwrap();
                                                if let Some(port_val) = property_map.get("port") {
                                                    Some(port_val.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                            .filter(|port_val| port_val.is_some())
                                            .map(|port_val| format!("{}:{}", cluster_host, port_val.unwrap().clone()))
                                            .collect();
        Ok(())
    }
}
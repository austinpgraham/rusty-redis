use std::{
    path::PathBuf,
    process::{Command, Stdio}, collections::HashSet, borrow::BorrowMut
};

use crate::{cluster::config::read_conf_file, local::pid::{get_currently_running_pids, PIDEntry, write_data_to_pid_file}};

#[inline]
fn spawn_server_process(conf_file: String) -> Result<u32, ()> {
    match Command::new("redis-server")
                                .arg(conf_file)
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn() {
                                    Ok(child) => Ok(child.id()),
                                    Err(msg) => {
                                        error!("{}", msg);
                                        Err(())
                                    }
                                }
}

#[inline]
fn spawn_cluster_process(cluster_host: &String, pid_entries: &HashSet<PIDEntry>) -> Result<u32, ()> {
    let cluster_string = pid_entries.iter()
                                        .map(|entry| format!("{}:{}", cluster_host, entry.port))
                                        .collect::<Vec<String>>();

    let mut root_command = Command::new("redis-cli");
    let mut command = root_command.borrow_mut();
    command = command.arg("--cluster").arg("create");
                    
    for entry in cluster_string.iter() {
        command = command.arg(entry);
    }

    match command.stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn() {
                    Ok(child) => Ok(child.id()),
                    Err(msg) => {
                        error!("{}", msg);
                        Err(())
                    }
                }                                
}

#[derive(Debug)]
struct ServerConf {
    pub conf_path: PathBuf,
    pub conf_port: String
}

pub fn start_cluster(cluster_host: &String, conf_files: Vec<String>) -> Result<(), String> {
    match get_currently_running_pids() {
        Ok(pids) => {
            if pids.len() != 0 {
                return Err("Servers are already running. If you wish to restart, first stop the cluster.".to_string());
            }
            let valid_processes: HashSet<PIDEntry> = conf_files.iter()
                                        .map(|path_str| PathBuf::from(path_str))
                                        .filter(|path| path.exists())
                                        .map(|file| {
                                            match read_conf_file(&file) {
                                                Ok(conf_content) => {
                                                    if let Some(port_val) = conf_content.get("port") {
                                                        Ok(ServerConf{ 
                                                            conf_path: file.clone(),
                                                            conf_port: port_val.clone()
                                                         })
                                                    } else {
                                                        Err(())
                                                    }
                                                },
                                                Err(_) => Err(())
                                            }
                                        })
                                        .filter(|file_result| file_result.is_ok())
                                        .map(|conf| {
                                            let conf_obj = conf.unwrap();
                                            let file_path = conf_obj.conf_path.into_os_string().into_string().unwrap_or(String::from("INVALID/"));
                                            match spawn_server_process(file_path.clone()) {
                                                Ok(child_pid) => {
                                                    info!("Process with conf {} successfully started with PID: {}.", file_path, child_pid);
                                                    Ok(PIDEntry{
                                                        port: conf_obj.conf_port.clone(),
                                                        pid: child_pid
                                                    })
                                                },
                                                Err(_) => {
                                                    error!("Process with conf {} failed to spawn.", file_path);
                                                    Err(())
                                                }
                                            }
                                        })
                                        .filter(|entry| entry.is_ok())
                                        .map(|entry| entry.unwrap())
                                        .collect();
            
            if valid_processes.len() <= 0 {
                Err("No valid configuration files were found.".to_string())
            } else {
                if let Err(msg) = write_data_to_pid_file(&valid_processes) {
                    error!("{}", msg);
                }

                match spawn_cluster_process(cluster_host, &valid_processes) {
                    Ok(_) => {
                        info!("Prmary cluster node started.");
                        Ok(())
                    },
                    Err(_) => Err("Failed to spawn main process for cluster.".to_string())
                }
            }
        },
        Err(_msg) => Err("Failed to assess current run state of system. Either manually delete $HOME/.rr/servers.pid or ensure all processes are stopped.".to_string())
    }
}
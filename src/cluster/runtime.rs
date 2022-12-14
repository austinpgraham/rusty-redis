use std::{
    borrow::BorrowMut,
    collections::HashSet,
    path::PathBuf,
    process::{Command, Stdio},
};

use mocktopus::macros::mockable;

use crate::{
    cluster::config::read_conf_file,
    local::pid::{get_currently_running_pids, write_data_to_pid_file, PIDEntry},
};

#[inline]
fn spawn_server_process(conf_file: String) -> Result<u32, ()> {
    match Command::new("redis-server")
        .arg(conf_file)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => Ok(child.id()),
        Err(msg) => {
            error!("{}", msg);
            Err(())
        }
    }
}

#[inline]
fn spawn_cluster_process(
    cluster_host: &String,
    pid_entries: &HashSet<PIDEntry>,
) -> Result<u32, ()> {
    let cluster_string = pid_entries
        .iter()
        .map(|entry| format!("{}:{}", cluster_host, entry.port))
        .collect::<Vec<String>>();

    let mut root_command = Command::new("redis-cli");
    let mut command = root_command.borrow_mut();
    command = command.arg("--cluster").arg("create");

    for entry in cluster_string.iter() {
        command = command.arg(entry);
    }

    match command.stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(child) => Ok(child.id()),
        Err(msg) => {
            error!("{}", msg);
            Err(())
        }
    }
}

#[inline]
fn kill_current_processes(pid_set: &HashSet<PIDEntry>) -> Result<(), ()> {
    let mut root_command = Command::new("kill");
    let mut command = root_command.borrow_mut();
    for pid in pid_set.iter() {
        command = command.arg(pid.pid.to_string());
    }

    match command.stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
        Ok(_) => Ok(()),
        Err(msg) => {
            error!("{}", msg);
            Err(())
        }
    }
}

#[derive(Debug)]
struct ServerConf {
    pub conf_path: PathBuf,
    pub conf_port: String,
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

pub fn stop_cluster() -> Result<(), String> {
    match get_currently_running_pids() {
        Ok(pids) => {
            if pids.len() <= 0 {
                return Err("No servers are running.".to_string());
            }

            match kill_current_processes(&pids) {
                Ok(_) => {
                    let empty_set: HashSet<PIDEntry> = HashSet::new();
                    write_data_to_pid_file(&empty_set)
                },
                Err(_) => Err("Failed to kill all server processes.".to_string())
            }
        },
        Err(_msg) => Err("Failed to assess current run state of system. Either manually delete $HOME/.rr/servers.pid or ensure all processes are stopped.".to_string())
    }
}

/// Spawns a Redis health check with a given endpoint in the format
/// host:port. Waits for the child process to finish and forwards the out.
///
/// # Arguments
/// * `health_endpoint` - host:port format endpoint used to check health.
///                       Should be an existing running Redis server.
///
/// # Examples
/// ```
/// let sample_endpoint = "127.0.0.1:7000".to_string();
/// spawn_health_check_process(&sample_endpoint).expect("Failed to spawn child health process.");
/// ```
#[mockable]
#[inline]
fn spawn_health_check_process(health_endpoint: &String) -> Result<(), String> {
    match Command::new("redis-cli")
        .arg("--cluster")
        .arg("check")
        .arg(health_endpoint)
        .spawn()
    {
        Ok(mut child) => match child.wait() {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Failed to run check command with error: {}", err);
                Err("Failed check command.".to_string())
            }
        },
        Err(_) => Err("Failed to spawn check process.".to_string()),
    }
}

/// Given a cluster host, get the currently running server processes and
/// use one to run a redis-cli --cluster check command.
///
/// # Arguments
/// * `cluster_host` - String representing the target cluster host.
///
/// # Examples
/// ```
/// let sample_host = "localhost".to_string();
/// check_cluster_health(&sample_host).expect("Failed to run server health check.");
/// ```
pub fn check_cluster_health(cluster_host: &String) -> Result<(), String> {
    match get_currently_running_pids() {
        Ok(pids) => {
            let pids_as_vector = pids.iter().collect::<Vec<&PIDEntry>>();
            let captain_pid = pids_as_vector.first();
            match captain_pid {
                Some(pid) => {
                    let health_endpoint = format!("{}:{}", cluster_host, pid.port);
                    spawn_health_check_process(&health_endpoint)
                }
                None => Err("There are no currently running server processes.".to_string()),
            }
        }
        Err(msg) => Err(msg),
    }
}

#[cfg(test)]
mod tests {

    use mocktopus::mocking::{MockResult, Mockable};

    use super::*;

    #[test]
    fn check_cluster_health_failed_pid_check() {
        get_currently_running_pids
            .mock_safe(|| MockResult::Return(Err("Failed to get PIDs".to_string())));

        let sample_host = "localhost".to_string();
        let cluster_result = check_cluster_health(&sample_host);
        assert!(cluster_result.is_err());
    }

    #[test]
    fn check_cluster_health_empty_pid_set() {
        get_currently_running_pids.mock_safe(|| MockResult::Return(Ok(HashSet::new())));

        let sample_host = "localhost".to_string();
        let cluster_result = check_cluster_health(&sample_host);
        assert!(cluster_result.is_err());
    }

    #[test]
    fn check_cluster_health_process_should_start() {
        get_currently_running_pids.mock_safe(|| {
            let mut test_set: HashSet<PIDEntry> = HashSet::new();
            test_set.insert(PIDEntry {
                port: "7000".to_string(),
                pid: 1234,
            });

            MockResult::Return(Ok(test_set))
        });

        spawn_health_check_process.mock_safe(|_| MockResult::Return(Ok(())));

        let sample_host = "localhost".to_string();
        let cluster_result = check_cluster_health(&sample_host);
        assert!(cluster_result.is_ok());
    }
}

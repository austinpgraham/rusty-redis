use std::{
    path::PathBuf,
    fs,
    io::{self, BufReader, BufRead, Write}, collections::HashSet, str::FromStr, hash::{Hasher, Hash}
};

use mocktopus::macros::mockable;

use crate::local::dotdir::get_or_create_local_config_dir;

// Name of the PID file used to store PIDs
// of servers currently running.
const SERVER_PID_FILE_NAME: &str = "servers.pid";

/// Create the PID file given the path.
/// 
/// # Arguments
/// * `dir` - The path destination for the PID file.
/// 
/// # Examples
/// ```
/// create_pid_file(PathBuf::from("./servers.pid")).expect("Failed to create file.");
/// ```
#[mockable]
#[inline]
fn create_pid_file(dir: &PathBuf) -> io::Result<fs::File> {
    fs::File::create(dir)
}

/// Open an existing PID file given it's target directory.
/// 
/// # Arguments
/// * `dir` - The path destination of the PID file.
/// 
/// # Examples
/// ```
/// open_pid_file(PathBuf::from("./servers.pid")).expect("Failed to open file.");
/// ```
#[mockable]
#[inline]
fn open_pid_file(dir: &PathBuf) -> io::Result<fs::File> {
    fs::File::open(dir)
}

/// Gets and creates if not exists the file used to store PIDs
/// of currently running Redis servers.
/// 
/// # Examples
/// ```
/// get_or_create_pid_file().expect("Failed to get PID file handler.");
/// ```
#[mockable]
fn get_or_create_pid_file() -> Result<fs::File, String> {
    let config_file_path = get_or_create_local_config_dir()?;
    let server_pid_path = config_file_path.join(SERVER_PID_FILE_NAME);
    if !server_pid_path.exists() {
        match create_pid_file(&server_pid_path) {
            Ok(file) => Ok(file),
            Err(_) => Err("Failed to create lock of server PIDs".to_string())
        }
    } else {
       match open_pid_file(&server_pid_path) {
        Ok(file) => Ok(file),
        Err(v) => {
            println!("{:?}", v);
            Err("Failed to open PID lock file.".to_string())
        }
       }
    }
}

/// Stores an entry in the PIDs file, hashed
/// by the port number to distinguish between the several 
/// servers that could be running at a time.
#[derive(Debug, PartialEq, Eq)]
pub struct PIDEntry {
    pub port: String,
    pub pid: u32
}

impl Hash for PIDEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pid.hash(state);
    }
}

/// Get a set of the running PIDs and the matching port numbers.
/// 
/// # Examples
/// ```
/// get_currently_running_pids().expect("Failed to get pids.");
/// ```
#[mockable]
pub fn get_currently_running_pids() -> Result<HashSet<PIDEntry>, String> {
    match get_or_create_pid_file() {
        Ok(file_handler) => {
            let reader = BufReader::new(file_handler);
            Ok(reader.lines()
                  .filter(|line| line.is_ok())
                  .map(|line| {
                        let line_str = line.unwrap();
                        let vector_entries: Vec<&str> = line_str.split_whitespace().collect();
                        if vector_entries.len() != 2 {
                            Err(())
                        } else {
                            let pid_result = <u32 as FromStr>::from_str(vector_entries[1]);
                            match pid_result {
                                Ok(pid) => Ok(PIDEntry {
                                    port: vector_entries[0].to_string(),
                                    pid: pid
                                }),
                                Err(_) => Err(())
                            }
                        }
                  })
                  .filter(|entry| entry.is_ok())
                  .map(|entry| entry.unwrap())
                  .collect::<HashSet<PIDEntry>>())
        },
        Err(msg) => Err(msg)
    }
}

/// Writes the given PIDs to a lock file, indicating that
/// there are currently node servers running.
/// 
/// # Arguments
/// * `pid_set` - Set of PIDs to write to the lock file.
/// 
/// # Examples
/// ```
/// let pid_set: HashSet<PIDEntry> = HashSet::new();
/// pid_set.insert(PIDEnry{ port: "7000", pid: 9 });
/// write_data_to_pid_file(pid_set).expect("Failed to write PIDs to save file.");
/// ```
#[mockable]
pub fn write_data_to_pid_file(pid_set: &HashSet<PIDEntry>) -> Result<(), String> {
    let config_file_path = get_or_create_local_config_dir()?;
    let server_pid_path = config_file_path.join(SERVER_PID_FILE_NAME);
    match create_pid_file(&server_pid_path) {
        Ok(mut file_handler) => {
            let file_str: String = pid_set.iter()
                                          .map(|entry| format!("{} {}\n", entry.port, entry.pid))
                                          .collect();
            
            match file_handler.write_all(file_str.as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err("Failed to write pid locks.".to_string())
            }
        },
        Err(_) => Err("Failed to create lock of PIDs.".to_string())
    }
}

#[cfg(test)]
mod tests {

    use mocktopus::mocking::{Mockable, MockResult};

    use super::*;

    #[test]
    fn test_get_or_create_pid_file_needs_creation_success() {
        let sample_path = PathBuf::from("./testdir");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));
        create_pid_file.mock_safe(move |_| MockResult::Return(fs::File::create("test.txt")));

        let file_result = get_or_create_pid_file();
        assert!(file_result.is_ok());

        fs::remove_file("test.txt").expect("Failed to remove temporary file.");
    }

    #[test]
    fn test_get_or_create_pid_file_needs_creation_failure() {
        let sample_path = PathBuf::from("./testdir");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));
        create_pid_file.mock_safe(|_| MockResult::Return(io::Result::Err(io::Error::from(io::ErrorKind::PermissionDenied))));

        let file_result = get_or_create_pid_file();
        assert!(file_result.is_err());
    }

    #[test]
    fn test_get_or_create_pid_file_already_exists_success() {
        let sample_path = PathBuf::from(".");
        let _ = fs::File::create(SERVER_PID_FILE_NAME).expect("Failed to create test file.");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));

        let file_result = get_or_create_pid_file();
        assert!(file_result.is_ok());

        fs::remove_file(SERVER_PID_FILE_NAME).expect("Failed to remove temporary file.");
    }

    #[test]
    fn test_get_or_create_pid_file_already_exists_failure() {
        let sample_path = PathBuf::from(".");
        let _ = fs::File::create(SERVER_PID_FILE_NAME).expect("Failed to create test file.");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));
        open_pid_file.mock_safe(|_| MockResult::Return(io::Result::Err(io::Error::from(io::ErrorKind::PermissionDenied))));

        let file_result = get_or_create_pid_file();
        assert!(file_result.is_err());

        fs::remove_file(SERVER_PID_FILE_NAME).expect("Failed to remove temporary file.");
    }

    #[test]
    fn test_get_currently_running_pids_success() {
        let test_path = PathBuf::from("./test/tests.pid");
        get_or_create_pid_file.mock_safe(move || MockResult::Return(Ok(fs::File::open(&test_path).expect("Failed to open test file."))));

        let pids = get_currently_running_pids();
        assert!(pids.is_ok());
        assert_eq!(pids.unwrap().len(), 3);
    }

    #[test]
    fn test_get_currently_running_pids_failure() {
        get_or_create_pid_file.mock_safe(|| MockResult::Return(Err("We failed....".to_string())));

        let pids = get_currently_running_pids();
        assert!(pids.is_err());
    }

    #[test]
    fn test_write_data_to_pid_file_fail_to_create() {
        let sample_path = PathBuf::from("./testdir");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));
        create_pid_file.mock_safe(|_| MockResult::Return(io::Result::Err(io::Error::from(io::ErrorKind::PermissionDenied))));

        let test_set: HashSet<PIDEntry> = HashSet::new();
        let write_result = write_data_to_pid_file(&test_set);
        assert!(write_result.is_err());
    }

    #[test]
    fn test_write_data_pid_file_success() {
        let sample_path = PathBuf::from(".");
        get_or_create_local_config_dir.mock_safe(move || MockResult::Return(Ok(sample_path.clone())));

        let mut test_set: HashSet<PIDEntry>  = HashSet::new();
        test_set.insert(PIDEntry { port: "7000".to_string(), pid: 1 });
        test_set.insert(PIDEntry { port: "7001".to_string(), pid: 2 });
        let write_result = write_data_to_pid_file(&test_set);
        assert!(write_result.is_ok());

        let pids = get_currently_running_pids();
        assert!(pids.is_ok());
        assert_eq!(pids.unwrap().len(), 2);
    }

}

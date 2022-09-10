use std::{
    path::PathBuf,
    fs,
    io
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

}

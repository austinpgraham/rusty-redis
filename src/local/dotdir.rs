use std::{
    path::{PathBuf, Path},
    env,
    fs, io
};

use mocktopus::macros::mockable;

// The hidden directory name of the directory
// to be used to store configuration items.
const CONFIG_DIR_NAME: &str = ".rr";

/// Get the directory stored as $HOME on the host
/// machine.
/// 
/// # Examples
/// ```
/// let system_home_dir = get_home_dir().expect("$HOME is not set.");
/// ```
#[mockable]
#[inline]
fn get_home_dir() -> Option<String> {
    env::var("HOME").ok()
}

/// Create a the home path system config directory, forwarding
/// the result onto the caller.
/// 
/// # Arguments
/// * `dir` - The directory to create.
/// 
/// # Examples
/// ```
/// let homepath = Path::new("/my/home/dir").join(CONFIG_DIR_NAME);
/// create_homepath_dir(&homepath).expect("Failed to create the hhomepath directory.")
/// ```
#[mockable]
#[inline]
fn create_homepath_dir(dir: &PathBuf) -> io::Result<()> {
    fs::create_dir(dir)
}

/// Get the path for, and create if needed, for the home config
/// directory given by CONFIG_DIR_NAME.
/// 
/// # Examples
/// ```
/// get_or_create_local_config_dir().expect("Failed to get or create the homepat config directory.");
/// ```
fn get_or_create_local_config_dir() -> Result<PathBuf, String> {
    match get_home_dir() {
        Some(dir) => {
            let home_path = Path::new(&dir).join(CONFIG_DIR_NAME);
            if !home_path.exists() {
                if create_homepath_dir(&home_path).is_err() {
                    return Err("Failed to create non-existent config directory .rr.".to_string())
                }
            }

            Ok(home_path)
        },
        None => Err("No value found for $HOME, cannot create CLI context.".to_string())
    }
}

#[cfg(test)]
mod tests {

    use mocktopus::mocking::{Mockable, MockResult};

    use super::*;

    #[test]
    fn test_get_or_create_local_config_dir_bad_home() {
        get_home_dir.mock_safe(|| MockResult::Return(None));

        let dir_result = get_or_create_local_config_dir();
        assert!(dir_result.is_err());
    }

    #[test]
    fn test_get_or_create_local_config_dir_home_exists() {
        get_home_dir.mock_safe(|| MockResult::Return(Some(".".to_string())));

        let dir_result = get_or_create_local_config_dir();
        assert!(dir_result.is_ok());

    }

    #[test]
    fn test_get_or_create_local_config_dir_home_doesnt_exist() {
        get_home_dir.mock_safe(|| MockResult::Return(Some("/nonexist".to_string())));
        create_homepath_dir.mock_safe(|_| MockResult::Return(io::Result::Ok(())));

        let dir_result = get_or_create_local_config_dir();
        assert!(dir_result.is_ok());
    }

}

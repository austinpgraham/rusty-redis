use std::{
    path::{PathBuf, Path},
    collections::HashMap,
    fs::File,
    io::{BufReader, BufRead},
    str::FromStr
};

pub const DEFAULT_BASE_CONF_PATH: &str = "/usr/local/etc/redis/cluster";

fn walk_directory(result_vec: &mut Vec<String>, base_dir: &Path) {
    if base_dir.is_dir() {
        if let Ok(read_dir) = base_dir.read_dir() {
            read_dir.for_each(|walk| {
                if let Ok(w) = walk {
                    walk_directory(result_vec, w.path().to_path_buf().as_ref());
                }
            });
        }
    } else if base_dir.is_file() {
        if let Some(ext) = base_dir.extension() {
            if ext.eq_ignore_ascii_case("conf") {
                if let Some(conf_file) = base_dir.as_os_str().to_str() {
                    result_vec.push(conf_file.to_string());
                }
            } 
        }
    }
}

pub fn aggregate_config_files(base_dir: &PathBuf) -> Result<Vec<String>, String> {
    if base_dir.is_dir() {
        let mut result_vec: Vec<String> = vec!();
        walk_directory(&mut result_vec, base_dir.as_path());
        Ok(result_vec)
    } else {
        Err(format!("Entry {} is not a directory.", base_dir.to_str().unwrap_or("DIR_ERROR")))
    }
}

pub fn list_conf_files(base_dir: &PathBuf) -> Result<(), String> {
    let conf_files = aggregate_config_files(base_dir);
    match conf_files {
        Ok(files) => {
            files.iter().for_each(|f| {
                info!("{}", f);
            });
            info!("Found {} configuration files.", files.len());
            Ok(())
        },
        Err(err) => Err(err)
    }
}

pub fn read_conf_file(file_path: &PathBuf) -> Result<HashMap<String, String>, String> {
    if file_path.exists() && file_path.is_file() && file_path.extension().unwrap().eq_ignore_ascii_case("conf") {
        match File::open(file_path) {
            Ok(file) => {

                // This is a very naive way of accomplishing the goal, but it'll
                // work for now
                let reader = BufReader::new(file);
                let mut result_map = HashMap::new();
                reader.lines().for_each(|line| {
                    if let Ok(l) = line {
                        let line_vec: Vec<String> = l.split_whitespace().map(|s| s.to_string()).collect();
                        if line_vec.len() == 2 {
                            result_map.insert(line_vec[0].clone(), line_vec[1].clone());
                        }
                    }
                });
                Ok(result_map)

            },
            Err(err) => Err(err.to_string())
        }
    } else {
        Err("Input entry is not a valid Redis configuration file.".to_string())
    }
}

#[inline]
pub fn resolve_base_file_path(base_dir: &Option<PathBuf>) -> PathBuf {
    match base_dir {
        Some(path) => path.clone(),
        None => PathBuf::from_str(DEFAULT_BASE_CONF_PATH).unwrap()
    }
}

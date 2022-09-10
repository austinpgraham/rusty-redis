use std::path::{PathBuf, Path};

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

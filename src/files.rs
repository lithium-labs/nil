use std::fs;
use std::path::PathBuf;
use crate::minimessage_const::ConstStr;
use std::env;

pub fn folder_size(path: &str) -> u64 {
    let mut size = 0;
    
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Some(path_str) = entry.path().to_str() {
                        size += folder_size(path_str);
                    }
                } else {
                    size += metadata.len();
                }
            }
        }
    }
    size
}

pub fn human_size(bytes: u64) -> ConstStr<16> {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut unit_idx = 0;
    let mut remainder = 0;

    while value >= 1024 && unit_idx < UNITS.len() - 1 {
        remainder = value % 1024;
        value /= 1024;
        unit_idx += 1;
    }

    let mut line = ConstStr::<16>::new();
    
    let fraction = (remainder * 100) / 1024;

    line.push_u64(value);
    if unit_idx > 0 {
        line.push_u8(b'.');
        if fraction < 10 { line.push_u8(b'0'); }
        line.push_u64(fraction);
    }
    
    line.push_u8(b' ');
    line.push_str(UNITS[unit_idx]);
    line
}

pub fn clear_dir(path: &str) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(metadata) = path.metadata() {
                if metadata.is_dir() {
                    if let Some(path_str) = path.to_str() {
                        clear_dir(path_str);
                    }
                    let _ = fs::remove_dir(&path);
                } else {
                    let _ = fs::remove_file(&path);
                }
            }
        }
    }
}

pub fn expand_path(input: ConstStr<260>) -> ConstStr<260> {
    let input_str = input.as_str();
    let mut result = PathBuf::new();

    if input_str.starts_with('~') {
        if let Ok(home_dir) = env::var("HOME") {
            result.push(home_dir);
        } else if cfg!(windows) {
            if let Ok(user_profile) = env::var("USERPROFILE") {
                result.push(user_profile);
            } else {
                result.push(".");
            }
        } else {
            result.push(".");
        }

        if input_str.len() > 1 {
            if input_str.starts_with("~/") {
                result.push(&input_str[2..]);
            } else if input_str.starts_with("~\\") {
                result.push(&input_str[2..]);
            } else {
                result.push(&input_str[1..]);
            }
        }
    } else {
        result.push(input_str);
    }

    let mut out = ConstStr::<260>::new();
    if let Some(path_str) = result.to_str() {
        out.push_str(path_str);
    }
    out
}

pub fn find_executable(name: &str) -> Option<ConstStr<260>> {
    if let Ok(path_env) = env::var("PATH") {
        for dir in path_env.split(if cfg!(windows) { ';' } else { ':' }) {
            let mut exe_path = PathBuf::from(dir);
            
            if cfg!(windows) {
                let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string());
                
                for ext in pathext.split(';') {
                    let ext = ext.strip_prefix('.').unwrap_or(ext);
                    
                    exe_path.push(name);
                    exe_path.set_extension(ext);

                    if exe_path.exists() {
                        if let Some(path_str) = exe_path.to_str() {
                            let mut result = ConstStr::<260>::new();
                            result.push_str(path_str);
                            return Some(result);
                        }
                    }

                    exe_path.pop(); 
                }
            } else {
                exe_path.push(name);
                if exe_path.exists() {
                    if let Some(path_str) = exe_path.to_str() {
                        let mut result = ConstStr::<260>::new();
                        result.push_str(path_str);
                        return Some(result);
                    }
                }
            }
        }
    }
    None
}
use std::fs;
use std::path::PathBuf;
use std::env;
use crate::{config::MAX_RULES, files::{expand_path, find_executable}, minimessage_const::ConstStr, ui::print_styled};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum DetectMethod {
    Binary(ConstStr<64>),
    EnvVar(ConstStr<64>),
    PathExists(ConstStr<260>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CleanMethod {
    RunCommand(ConstStr<64>, ConstStr<128>),
    CleanPath(ConstStr<260>),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CacheRule {
    pub name: ConstStr<32>,
    pub detect: DetectMethod,
    pub clean: CleanMethod,
    pub size_dir: Option<ConstStr<260>>
}

fn get_exe_dir() -> ConstStr<260> {
    let mut path = ConstStr::<260>::new();
    
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            if let Some(parent_str) = parent.to_str() {
                path.push_str(parent_str);
            }
        }
    }
    
    if path.len == 0 {
        path.push_str(".");
    }
    
    path
}

fn read_rules_file(path: &ConstStr<260>) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path.as_str())
}

pub fn load_rules(out_rules: &mut [CacheRule; MAX_RULES]) -> usize {
    let exe_dir = get_exe_dir();

    let mut rules_path = PathBuf::from(exe_dir.as_str());
    rules_path.push("rules.txt");

    let rules_str = ConstStr::from(rules_path.to_str().unwrap_or("rules.txt"));
    
    let file_content = match read_rules_file(&rules_str) {
        Ok(content) => content,
        Err(_) => {
            // Try templates folder
            let os_str = match std::env::consts::OS {
                "linux" => "linux",
                "macos" => "macos",
                "windows" => "windows",
                _ => {
                    print_styled("<red>Error: <gray>Unsupported OS.");
                    std::process::exit(1);
                }
            };

            let mut templates_path = PathBuf::from(exe_dir.as_str());
            templates_path.push("templates");

            if !templates_path.exists() {
                print_styled("<red>Error: <gray>templates folder not found.");
                std::process::exit(1);
            }

            templates_path.push(os_str);
            templates_path.push("rules.txt");

            match read_rules_file(&ConstStr::from(templates_path.to_str().unwrap_or("rules.txt"))) {
                Ok(content) => content,
                Err(_) => {
                    print_styled("<red>Error: <gray>Could not read rules.txt from templates/" + os_str + "/");
                    std::process::exit(1);
                }
            }
        }
    };

    let content = match std::str::from_utf8(&file_content) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let mut count = 0;
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());

    while let Some(name) = lines.next() {
        if count >= MAX_RULES { break; }

        let det_line = lines.next().unwrap_or("");
        let detect = match det_line.as_bytes().get(0) {
            Some(b'1') => DetectMethod::Binary(ConstStr::from(&det_line[1..])),
            Some(b'2') => DetectMethod::EnvVar(ConstStr::from(&det_line[1..])),
            _ => DetectMethod::PathExists(ConstStr::from(&det_line[1..])),
        };

        let clean_line = lines.next().unwrap_or("");
        let clean = match clean_line.as_bytes().get(0) {
            Some(b'1') => {
                let cmd_part = &clean_line[1..];
                if let Some(idx) = cmd_part.find(';') {
                    CleanMethod::RunCommand(
                        ConstStr::from(&cmd_part[..idx]),
                        ConstStr::from(&cmd_part[idx+1..]),
                    )
                } else {
                    CleanMethod::RunCommand(
                        ConstStr::from(cmd_part),
                        ConstStr::new()
                    )
                }
            },
            _ => CleanMethod::CleanPath(ConstStr::from(&clean_line[1..])),
        };

        let size_dir = lines.next().map(|s| {
            if s.is_empty() { None } else { Some(ConstStr::from(s)) }
        }).flatten();

        out_rules[count] = CacheRule {
            name: ConstStr::from(name),
            detect,
            clean,
            size_dir,
        };
        count += 1;
    }

    count
}

pub fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

pub fn is_rule_active(rule: &CacheRule) -> bool {
    match &rule.detect {
        DetectMethod::Binary(name) => find_executable(name.as_str()).is_some(),
        DetectMethod::EnvVar(key) => {
            env::var(key.as_str()).is_ok()
        },
        DetectMethod::PathExists(path) => {
            let expanded = expand_path(*path);
            path_exists(expanded.as_str())
        }
    }
}
#![feature(const_trait_impl)]
#![feature(const_cmp)]

use std::{env, process::exit};

use crate::{
    r#abstract::Timer, cli::{Commands, find_suggestion}, config::MAX_RULES, files::{clear_dir, expand_path, folder_size, human_size}, minimessage_const::{ConstStr, serialize}, rules::{CacheRule, CleanMethod, DetectMethod, is_rule_active, load_rules}, ui::{print_styled, println}
};

mod cli;
mod files;
mod minimessage_const;
mod r#abstract;
mod rules;
mod ui;
mod config;

const HELP_MESSAGE: &str = serialize::<1024>(concat!(r#"<#55AAFF>nil</#55AAFF> <white>v"#, env!("CARGO_PKG_VERSION"), r#"</white>
<#A5FAFF>"#, env!("CARGO_PKG_DESCRIPTION"), r#"</#A5FAFF>

<white><b><u>Usage:</u></b> <#55AAFF><b>nil</b> \<COMMAND> [OPTION]</#55AAFF>

<white><b><u>Commands:</u></b></white>
  <b>s, scan </b>        Scans the caches and shows the sizes of them.
  <b>c, clean</b>        Cleans the caches.
  <b>l, list </b>        Lists caches found.
  <b>help    </b>        Prints the help message.

<b><u>Options:</u></b>
  <b>-h, --help   </b>   Print help
  <b>-v, --version</b>   Print version

<b><u>Subcommand Options:</u></b>
  <b>clean:</b>
    <b>--unsafe, -u</b>  Cleans the cache directory instead of using the preferred
                  method.
"#)).as_str();

fn print_help() {
    println(HELP_MESSAGE);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let cmd = if args.len() > 1 {
        &args[1]
    } else {
        "help"
    };

    let is_unsafe = args.contains(&"--unsafe".to_string()) || args.contains(&"-u".to_string());

    let command = match cmd {
        "scan" | "s" => Commands::Scan,
        "clean" | "c" => Commands::Clean,
        "list" | "l" => Commands::List,
        "help" | "-h" | "--help" => Commands::Help,
        "version" | "-v" | "--version" => {
            println(env!("CARGO_PKG_VERSION"));
            Commands::Exit
        }
        _ => {
            print_styled("<red>Error: <white>Unknown command.");
            if let Some(suggestion) = find_suggestion(cmd) {
                let mut msg = ConstStr::<256>::from("  Did you mean '<yellow>");
                msg.push_str(suggestion);
                msg.push_str("<white>'?");
                print_styled(&msg);
            }

            Commands::Exit
        }
    };

    const BLANK_RULE: CacheRule = CacheRule {
        name: ConstStr::new(),
        detect: DetectMethod::PathExists(ConstStr::new()),
        clean: CleanMethod::CleanPath(ConstStr::new()),
        size_dir: None,
    };

    let mut rules: [CacheRule; MAX_RULES] = [BLANK_RULE; MAX_RULES];
    let rule_count = load_rules(&mut rules);

    match command {
        Commands::Exit => {
            std::process::exit(1);
        }
        Commands::Help => {
            print_help();
        }
        Commands::List => {
            println("Detected tools:");

            for i in 0..rule_count {
                let r = &rules[i];
                if is_rule_active(r) {
                    let mut s = ConstStr::<64>::from("  <gray>- <yellow>");
                    s.push_str(r.name.as_str());
                    
                    print_styled(s.as_str()); 
                }
            }
        }
        Commands::Scan => {
            let mut total: u64 = 0;
            
            print_styled("<white>Tool Sizes:\n");

            for i in 0..rule_count {
                let t = &rules[i];
                if !is_rule_active(t) { continue; }

                let size = match &t.size_dir {
                    Some(dir) => {
                        let s = folder_size(expand_path(*dir).as_str());
                        total += s;
                        s
                    }
                    None => 0,
                };

                let mut line = ConstStr::<256>::new();
                line.push_str("  ");

                if size == 0 {
                    line.push_str("<#555555>");
                } else {
                    line.push_str("<#55AAFF>");
                }
                
                line.push_str(t.name.as_str());
                line.push_str(": <white>");
                line.push_str(human_size(size).as_str());
                
                print_styled(line.as_str());
            }

            let mut final_msg = ConstStr::<128>::new();
            final_msg.push_str("\n<#55AAFF>Total: <white>");
            final_msg.push_str(human_size(total).as_str());
            print_styled(final_msg.as_str());
        }
        Commands::Clean => {
            let start = Timer::now();
            let mut total: u64 = 0;

            let mut msg = ConstStr::<256>::new();
            let mut count = 0;

            for tool in rules {
                if !is_rule_active(&tool) {
                    continue;
                }

                count += 1;

                msg.clear();
                msg.push_str("<yellow>Cleaning cache of ");
                msg.push_str(&tool.name);
                msg.push_str("...");
                print_styled(&msg);

                match &tool.size_dir {
                    Some(k) => {
                        let size = folder_size(&expand_path(*k));
                        total += size;
                    }
                    None => {}
                }

                let clean = if is_unsafe && tool.size_dir.is_some() {
                    CleanMethod::CleanPath(tool.size_dir.unwrap())
                } else {
                    tool.clean.clone()
                };

                match &clean {
                    CleanMethod::RunCommand(cmd, args) => {
                        msg.clear();
                        msg.push_str("<#D4D4D4>Running command: <gray>");
                        msg.push_str(cmd);
                        msg.push_u8(b' ');
                        msg.push_str(args);
                        print_styled(&msg);

                        if let Err(code) = crate::r#abstract::run_command(cmd, args) {
                            msg.push_u64(code as u64);
                            msg.push_u8(b')');
                            print_styled(&msg);
                        }
                    }
                    CleanMethod::CleanPath(path) => {
                        let dir = expand_path(*path);

                        let mut escaped_dir = ConstStr::<512>::new(); 
                        for b in dir.as_str().bytes() {
                            if b == b'\\' {
                                escaped_dir.push_str("\\\\");
                            } else {
                                escaped_dir.push_u8(b);
                            }
                        }

                        msg.clear();
                        msg.push_str("<#D4D4D4>Clearing directory: <gray>\"");
                        msg.push_str(&escaped_dir);
                        msg.push_str("\"");
                        print_styled(&msg);

                        let _ = clear_dir(&dir);
                    }
                }
            }

            msg.clear();
            msg.push_str("<green>Done! Cleaned ");
            msg.push_u64(count);
            msg.push_str(" caches <dark_green>(");
            msg.push_str(&human_size(total));
            msg.push_str(")</dark_green>. Took ");
            msg.push_u64(start.elapsed_ms());
            msg.push_str(" ms.");
            print_styled(&msg);
        }
    }

    exit(0); // i got no idea why it doesnt exit normally
}

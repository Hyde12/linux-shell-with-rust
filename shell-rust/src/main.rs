#[allow(unused_imports)]
use std::io::{self, Write};
use is_executable::IsExecutable;
use std::{env::{self, current_dir, set_current_dir}, fs::{self, read_to_string}, path::Path, process::{Command, Output}, str::Split};

fn extract_path() -> String {
    match env::var("PATH") {
        Ok(path) => return path,
        Err(_e) => return current_dir().unwrap().display().to_string(),
    }
}

fn extract_home() -> String {
    match env::var("HOME") {
        Ok(path) => return path,
        Err(_e) => return current_dir().unwrap().display().to_string(),
    }
}

fn check_dir(file: &str, directories: Vec<&str>) -> String {
    for i in directories {
        let file_path = format!("{}/{}", i, file);
        if let Ok(metadata) = fs::metadata(file_path) {
            if metadata.is_file() {
                return i.to_string() + "/" + file;
            }
        }
    }
    
    "".to_string()
}

fn run_file(directory: &Path, args: &Vec<String>) {
    let output: Output;

    io::stdout().flush().unwrap();

    output = Command::new(directory)
        .args(args)
        .output()
        .expect("error: failed to execute");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn check_backslash(text: &str, quote_kind: &str) -> String {
    let mut new_i = String::new();
    
    if text.contains('\\') {
        let special = ['\\', '$', '\"', 'n'];

        let mut exited = false;
    
        for j in text.chars() {
            if exited {
                if special.contains(&j) {
                   let _ = new_i.pop(); 
                }
                exited = false;
            } else if j == '\\'  {
                exited = true;
            } else {
                if j.to_string() == quote_kind.to_string() {
                    
                    continue;
                }
                
            }
            new_i.push(j);
            
        }
    } else {
        new_i = text.to_string();
    }

    new_i
}

// ECHO
fn extract_all_quotes(cmd: &mut Split<'_, char>, first_ln: &str, quote_kind: &str, backslash_rules: bool) -> String {
    let mut to_print = "".to_string();

    if first_ln.ends_with(&quote_kind) && (!first_ln.contains("\\") || quote_kind != "\"" || !backslash_rules) { 
        to_print += &format!("{} ", first_ln.replacen(&quote_kind, "", 2));
        return to_print;
    } else { 
        if quote_kind == "\"" { to_print += &format!("{} ", check_backslash(&first_ln.replacen(&quote_kind, "", 1), quote_kind));
                            } else { to_print += &format!("{} ", &first_ln.replacen(&quote_kind, "", 1)); }
    }
    
    while let Some(i) = cmd.next() {
        let new_i = if !backslash_rules { i.to_string() } else { check_backslash(i, quote_kind) };

        if new_i.starts_with(&quote_kind) {
            break;
        } else if new_i.ends_with(&quote_kind) {
            to_print += &format!("{} ", new_i.trim_end_matches(&quote_kind));
            break;
        } else {
            to_print += &format!("{new_i} ");
        }
    }

    to_print
}

fn extract_valid_quotes(cmd: &mut Split<'_, char>, first_ln: &str) -> String {
    let mut exited: bool = false;
    let mut to_print: String;
    
    if first_ln.contains("\\") {
        exited = true;
        to_print = format!("{} ", first_ln.replace("\\", "")).to_string();
    } else {
        to_print = format!("{} ", first_ln).to_string();
    }
    
    for i in cmd.clone() {
        if i != "" || exited { 
            if i.contains("\\") {
                to_print += &format!("{} ", i.replace("\\", ""));
                exited = true;
                continue;
            }
            to_print += &format!("{i} ") ;
            exited = false;
        } 
    }

    to_print
}

fn command_not_found(input: &str, command: &str) {
    if command == "standard" {
        println!("{}: command not found", input)
    } else if command == "type" {
        println!("{}: not found", input)
    } else if command == "cd" {
        println!("cd: {}: No such file or directory", input)
    }
}

fn check_quote(text: &str) -> &str {
    if text.starts_with("'") { "'" } 
    else if text.starts_with('"') { "\"" } 
    else { "" }
}

fn command(input: &str, path: &str) -> bool {
    let valid_shell_commands = ["exit", "echo", "type", "pwd", "cd"];
    let mut cmd = input.split(' ');
    let first_cmd = cmd.next().unwrap();

    match first_cmd {
        "exit" => {
            return false
        }
        "echo" => { 
            let mut to_print = String::new();
        
            while let Some(arg) = cmd.next() {
                
                let quote_kind: &str;
                
                if arg.starts_with("'") { quote_kind =  "'"; } 
                else if arg.starts_with('"') { quote_kind = "\""; } 
                else if arg == "" { continue; }
                else { quote_kind = ""; }

                if !quote_kind.is_empty() {
                    let quoted_arg = extract_all_quotes(&mut cmd, arg, quote_kind, true);
                    to_print.push_str(&quoted_arg.trim());
                } else {
                    let non_quoted_arg = extract_valid_quotes(&mut cmd, arg).trim().to_string();
                    to_print.push_str(&non_quoted_arg);
                    break;
                }
                
                to_print.push_str(" ");
            }
            println!("{}", to_print.trim());
        }
        "type" => { 
            let type_command: &str = cmd.next().unwrap_or("");

            if valid_shell_commands.contains(&type_command) 
            {
                println!("{} is a shell builtin", type_command);
            } 
            else if path != "" 
            {
                let directory = check_dir(type_command, path.split(':').collect()); // SEMI COLON if windows, COLON if linux
                if  directory != "" {
                    println!("{}", directory);
                } else {
                    command_not_found(type_command, first_cmd);
                }
            } 
            else 
            {
                command_not_found(input.trim_start_matches("type "), first_cmd);
            }
        }
        "pwd" => println!("{}", current_dir().unwrap().display()),
        "cd" => {
            let temp_dir: &str = cmd.next().unwrap_or("");
            if let Ok(metadata) = fs::metadata(temp_dir) {
                if metadata.is_dir() {
                    let _ = set_current_dir(temp_dir.to_string());  
                }
            } else if temp_dir == "~" {
                let _ = set_current_dir(extract_home());  
            } else {
                command_not_found(temp_dir, "cd");
            }
        }
        _ => {
            let x = check_quote(&first_cmd);
            let path_string;
            let mut dirs: Vec<&str> = path.split(':').collect();
            
            let current_dir = current_dir().unwrap().display().to_string();
            dirs.push(&current_dir);

            if x != "" {
                path_string = check_dir(&extract_all_quotes(&mut cmd, first_cmd, x, false).trim(), path.split(':').collect());
            } else {
                path_string = check_dir(first_cmd, dirs);
            }


            let directory = Path::new(&path_string);
            
            
            if &directory.is_file() == &true {
                let mut args: Vec<String> = Vec::new();

                while let Some(arg) = cmd.next() {
                    let quote_kind: &str;
                    
                    quote_kind = check_quote(arg);

                    if !quote_kind.is_empty() {
                        let quoted_arg = extract_all_quotes(&mut cmd, arg, quote_kind, false);
                        args.push(quoted_arg.trim().to_string());
                    } else {
                        let non_quoted_arg = extract_valid_quotes(&mut cmd, arg).trim().to_string();
                        args.push(non_quoted_arg.trim().to_string());
                        break;
                    }
                }
                
                if directory.is_executable() { 
                    run_file(&directory, &args);
         
                } else {
                    for i in args.iter() {
                        if let Ok(metadata) = fs::metadata(i) {
                            if metadata.is_file() {
                                print!("{}", read_to_string(i).expect("fail!"));
                            }
                        }
                    }
                }
            } else {
                command_not_found(input, "standard");
            }

        }
    }

    return true
}

fn main() {
    let path: String = extract_path();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        
        let input = input.trim_end();
        
        println!("");
        if !command(input, &path) {
            break;
        }
        println!("");
    }
    
}
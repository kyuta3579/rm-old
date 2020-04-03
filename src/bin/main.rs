/*rm-old: remove the old files in dir.
Usage       :rm-old [dir_path] [option]
Options
-r              : recursion. target dir in dir.
-i              : ask each file when remove.
-y              : assume yes.
-d [days]       : specify duration of day.(default is 60 days)
-v              : verbose.
-n              : dry run. not a remove, only show log.
--remove-dir    : remove directory.
--remove-empty  : remove empty dir.
-h, --help      : show help.
*/

extern crate rm_old;

use rm_old::{arg::Config, fs::Dir};

use std::env;
use std::path::Path;
use std::fs;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    let ret_config = match Config::parse_config(&args) {
        Ok(config)    => config,
        Err(err_msg)      => {
            println!("{}\nUsage: rm-old [dir_path] [option]", err_msg);
            return ;
        },
    };

    ret_config.print();

    let target_files = match Dir::get_target_files(&ret_config) {
        Ok(files)       => files,
        Err(err_msg)    => {
            println!("{}", err_msg);
            return ;
        }
    };

    if target_files.is_empty() {
        println!("Target files not exists!");
        return ;
    }

    match execute_rm(&target_files, &ret_config) {
        Ok(_)           => println!("Complete!"),
        Err(err_msg)    => println!("{}", err_msg),
    }
}

fn execute_rm(target_dirs: &Vec<Dir>, config: &Config) -> Result<(), String> {
    let mut amount_target = 0;
    for dir in target_dirs.iter().rev() {
        dir.print();
        amount_target = amount_target + dir.get_amount_files();
    }

    println!("target files: {}", amount_target);

    match interaction("Remove the above files. Ok? [Y/n]: ", config.assume_yes()) {
        Ok(_)   => {},
        Err(_)  => {return Err("Canceled.".to_string());}
    }

    for dir in target_dirs.iter().rev() {
        println!("{}/ :", dir.get_parent_path());
        for f in dir.get_files().iter() {
            match remove_target(&format!("{}/{}", dir.get_parent_path(), f), config) {
                Ok(_)   => {},
                Err(err_msg)  => {
                    println!("{} {}", err_msg, f);
                    continue;
                },
            }
        }

        if fs::read_dir(dir.get_parent_path()).unwrap().next().is_none(){
            if !config.dry_run() && config.remove_empty() {
                match fs::remove_dir(dir.get_parent_path()) {
                    Ok(()) => println!("Removed: {}", dir.get_parent_path()),
                    Err(_) => println!("Fatal Error."),
                }
            } else if config.dry_run() && config.remove_empty() {
                println!("Removed: {}", dir.get_parent_path());
            }
        }
        println!("");
    }
    Ok(())
}

fn remove_target(file_path: &String, config: &Config) -> Result<(), String>{

    if config.do_intr() {
        println!("    {:?}", Path::new(file_path).file_name().unwrap());
        match interaction("Remove This file? [Y/n]: ", config.assume_yes()) {
            Ok(_)   => {},
            Err(_)  => {return Err("Canceled:".to_string());}
        }
    }
    if !config.dry_run() {
        if !config.remove_dir() {
            match fs::remove_file(file_path) {
                Ok(_)   => {println!("Removed: {:?}", Path::new(file_path).file_name().unwrap())},
                Err(_)  => {return Err("Remove failed:".to_string());},
            }
        } else {
            match fs::remove_dir_all(file_path) {
                Ok(_)   => {println!("Removed: {:?}", Path::new(file_path).file_name().unwrap())},
                Err(_)  => {return Err("Remove failed:".to_string());},
            }
        }
    } else {
        println!("Removed: {}", file_path);
    }

    if config.do_intr() {
        println!("");
    }

    Ok(())
}

fn interaction(msg: &str, assume_yes: bool) -> Result<(), ()> {
    let mut _ret: Result<(), ()> = Ok(());

    if assume_yes {
        return _ret;
    }

    print!("{}", msg);
    io::stdout().flush().unwrap();

    loop{
        let s = get_string().unwrap().as_str().chars().nth(0).unwrap();
        match s {
            'Y' => {
                _ret = Ok(());
                break;
            }
            'n' => {
                _ret = Err(());
                break;
            }
            _   => {
                print!("Invalid value. Please input [Y/n].\n{}", msg);
                io::stdout().flush().unwrap();
            },
        }
    }
    _ret
}

fn get_string() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf)
}
/*rm-old: remove the old files in dir.
Usage       :rm-old [dir_path] [option]
Options
-r          : recursion. target dir in dir.
-i          : ask each file when remove.
-y          : assume yes.
-d [days]   : specify duration of day.(default is 60 days)
-v          : verbose.
-n          : dry run. not a remove, only show log.
-h, --help  : show help.
*/

use std::env;
use std::path::Path;
use std::time::SystemTime;
use std::fs::{self, File};
use std::io::{self, Write};

struct Dir{
    parent_path:    String,
    files_path:     Vec<String>,
}

impl Dir {
    fn new() -> Dir {
        Dir {
            parent_path:    String::new(),
            files_path:     Vec::new(),
        }
    }

    fn get_target_files(config: &Config) -> Result<Vec<Dir>, String> {
        let now_sys_time            = SystemTime::now();
        let mut targets: Vec<Dir>    = Vec::new();

        for path in config.target_path.iter() {
            let mut t = match get_files_in_dir(path, config, now_sys_time) {
                Ok(files)       => files,
                Err(err_msg)    => return Err(err_msg),
            };
            targets.append(&mut t);
        }

        Ok(targets)
    }

    fn print(&self) -> () {
        println!("{}/:",self.parent_path);
        for file in self.files_path.iter().rev() {
            println!("    {}", file);
        }
        println!("");
    }

    fn get_amount_files(&self) -> u64 {
        self.files_path.len() as u64
    }
}

struct Config {
    target_path:    Vec<String>,
    duration_days:  u64,
    change_days:    bool,
    do_intr:        bool,
    assume_yes:     bool,
    recursion:      bool,
    verbose:        bool,
    dry_run:        bool,
}

impl Config {
    fn new () -> Config {
        Config {
            target_path:    Vec::new(),
            duration_days:  60,
            change_days:    false,
            do_intr:        false,
            assume_yes:     false,
            recursion:      false,
            verbose:        false,
            dry_run:        false,
        }
    }

    fn print(&self) -> () {
        if self.verbose {
            for path in self.target_path.iter() {
                println!("target_path   : {}", path);
            }
            println!("duration_days : {}", self.duration_days);
            if self.do_intr {
                println!("interaction   : yes");
            } else {
                println!("interaction   : no");
            }
            if self.assume_yes {
                println!("assume_yes    : yes");
            } else {
                println!("assume_yes    : no");
            }
            if self.recursion {
                println!("recursion     : yes");
            } else {
                println!("recursion     : no");
            }
            if self.dry_run {
                println!("dry_run       : yes");
            } else {
                println!("dry_run       : no");
            }
            println!("");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let ret_config = match parse_config(&args) {
        Ok(config)    => config,
        Err(err_msg)      => {
            println!("{}\nUsage: rm-old [dir_path] -d [days] -iryv", err_msg);
            return ;
        },
    };

    ret_config.print();

    let target_files = match Dir::get_target_files(&ret_config) {
        Ok(files)       => files,
        Err(err_msg)    => {
            println!("{:?}", err_msg);
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

fn parse_config(args: &[String]) -> Result<Config, String> {
    let mut config = Config::new();

    for arg in args[1..].iter() {
        if "--help" == arg || "-h" == arg {
            return Err(show_help());
        } else if !config.change_days && '-' == arg.as_str().chars().nth(0).unwrap() {
            match get_option(arg, &mut config) {
                Ok(_)  => {},
                Err(err_msg)          => {return Err(err_msg);},
            }
        } else {
            if config.change_days {
                let days = arg.parse::<u64>();
                match days {
                    Ok(num)   => {
                        config.duration_days = num;
                        config.change_days = false;
                    },
                    Err(_)    => {
                        return Err(format!("rm-old -d: Illegal value: {}", arg));
                    },
                }
            } else {
                match get_path(arg, &mut config){
                    Ok(_)  => {},
                    Err(err_msg)          => {return Err(err_msg);},
                }
            }
        }
    }

    if config.change_days {
        return Err("rm-old -d: Input duration days after -d.".to_string());
    } else if config.target_path.is_empty(){
        config.target_path.push(".".to_string());
    }
    Ok(config)
}

fn get_option(arg: &String, config: &mut Config) -> Result<(), String> {
    for c in arg.as_str()[1..].chars() {
        match c {
            'd' => {
                config.change_days = true;
            },
            'r' => {
                config.recursion = true;
            },
            'i' => {
                config.do_intr = true;
            },
            'y' => {
                config.assume_yes = true;
            },
            'v' => {
                config.verbose = true;
            },
            'n' => {
                config.dry_run = true;
            }
            _ => {
                return Err(format!("rm-old: illegal option: {}", c));
            },
        }
    }

    Ok(())
}

fn get_path(arg: &String, config: &mut Config) -> Result<(), String> {
    let path = Path::new(arg);
    if path.exists() && path.is_dir(){
        if path.has_root() {
            config.target_path.push(arg.clone());
        } else {
            if arg.chars().last().unwrap() == '/' {
                let mut arg_clone = arg.clone();
                arg_clone.remove(arg.len()-1);
                config.target_path.push(format!("./{}", arg_clone));
            } else {
                config.target_path.push(format!("./{}", arg));
            }
        }
    } else {
        return Err(format!("rm-old: illegal path: {}", arg));
    }

    Ok(())
}

fn get_files_in_dir(path: &String, config: &Config, now: SystemTime) -> Result<Vec<Dir>, String> {
    let mut target: Vec<Dir> = Vec::new();
    let mut target_dir = Dir::new();

    let files = match fs::read_dir(path) {
        Err(why)    => {
            return Err(format!("Can not open dir: {:?}", why.kind()));
        },
        Ok(paths)   => paths,
    };

    target_dir.parent_path = path.clone();

    for f in files {
        let file_path           = f.unwrap().path();
        let file_meta           = File::open(&file_path).unwrap().metadata().unwrap();
        let duration_time_by_ac = match now.duration_since(file_meta.accessed().unwrap()){
            Ok(duration)    => duration.as_secs(),
            Err(e)          => {
                return Err(format!("Clock may have gone backwards: {:?}",e.duration()));
            },
        };

        if file_meta.is_file() && (config.duration_days * 86400 < duration_time_by_ac) {
                target_dir.files_path.push(
                    file_path.as_path().file_name().unwrap().to_str().unwrap().to_string());
        } else if file_meta.is_dir() && config.recursion {
            match get_files_in_dir(
                &(file_path.as_path().to_str().unwrap().to_string()), config, now)
            {
                Ok(mut files)       => {
                    target.append(&mut files);
                },
                Err(err_msg)    => return Err(err_msg),
            };
        }
    }
    if !target_dir.files_path.is_empty() {
        target.push(target_dir);
    }
    Ok(target)
}


fn execute_rm(target_dirs: &Vec<Dir>, config: &Config) -> Result<(), String> {
    let mut amount_target = 0;
    for dir in target_dirs.iter().rev() {
        dir.print();
        amount_target = amount_target + dir.get_amount_files();
    }

    println!("target files: {}", amount_target);

    match interaction("Remove the above files. Ok? [Y/n]: ", config.assume_yes) {
        Ok(_)   => {},
        Err(_)  => {return Err("Canceled.".to_string());}
    }

    for dir in target_dirs.iter().rev() {
        println!("{}/ :", dir.parent_path);
        for f in dir.files_path.iter() {
            match remove_target(&format!("{}/{}", dir.parent_path, f), config) {
                Ok(_)   => {},
                Err(err_msg)  => {
                    println!("{} {}", err_msg, f);
                    continue;
                },
            }
        }
        println!("");
    }
    
    Ok(())
}

fn remove_target(file_path: &String, config: &Config) -> Result<(), String>{

    if config.do_intr {
        println!("    {:?}", Path::new(file_path).file_name().unwrap());
        match interaction("Remove This file? [Y/n]: ", config.assume_yes) {
            Ok(_)   => {},
            Err(_)  => {return Err("Canceled:".to_string());}
        }
    }
    if !config.dry_run {
        match fs::remove_file(file_path) {
            Ok(_)   => {println!("Removed: {:?}", Path::new(file_path).file_name().unwrap())},
            Err(_)  => {return Err("Remove failed:".to_string());},
        }
    } else {
        println!("Removed: {}", file_path);
    }

    if config.do_intr {
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

fn show_help() -> String {
    "rm-old: remove the old files in dir.
    -r          : recursion. target dir in dir.
    -i          : ask each file when remove.
    -y          : assume yes.
    -d [days]   : specify duration of day.(default is 60 days)
    -v          : verbose.
    -n          : dry run. not a remove, only show log.
    -h, --help  : show help.(this!)
    ".to_string()
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_get_option() {
        let options = ['r', 'i', 'y', 'v', 'n'];
        let mut config = Config::new();

        for c in options.iter() {
            assert!(get_option(&format!("-{}", c).to_string(), &mut config).is_ok());
        }

        assert!(config.recursion);
        assert!(config.do_intr);
        assert!(config.assume_yes);
        assert!(config.verbose);
        assert!(config.dry_run);

        assert!(get_option(&format!("-{}", 'e').to_string(), &mut config).is_err());

    }

    #[test]
    fn test_parse_config() {
        let correct_args: Vec<Vec<String>> = vec![  vec!["rm-old".to_string(), "-d".to_string(), "90".to_string(), "-riyvn".to_string()],
                                                    vec!["rm-old".to_string(), "-d".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string(), "-riyvn".to_string()],
                                                    vec!["rm-old".to_string(), "-driyvn".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string()],
        ];

        let invalid_args: Vec<Vec<String>> = vec![  // After "-d" is not number.
                                                    vec!["rm-old".to_string(), "-d".to_string(), "-riyvn".to_string()],
                                                    // not path.
                                                    vec!["rm-old".to_string(), "jifsl.?s_sdfe".to_string()],
                                                    // After "-d"  not exist.
                                                    vec!["rm-old".to_string(), "-driyvn".to_string()],
        ];

        for arg in correct_args.iter() {
            assert!(parse_config(&arg).is_ok());
        }
        for arg in invalid_args.iter() {
            assert!(parse_config(&arg).is_err());
        }
    }
    #[test]
    fn test_get_files() {
        let mut config = Config {
            target_path:    vec!["test_dir".to_string()],
            duration_days:  0,
            change_days:    false,
            do_intr:        false,
            assume_yes:     false,
            recursion:      false,
            verbose:        false,
            dry_run:        false,
        };

        let mut test_dir = match Dir::get_target_files(&config) {
            Ok(dir)     => dir,
            Err(msg)    => panic!("{}", msg),
        };
        assert_eq!(1, test_dir.len());

        config.recursion = true;

        test_dir = match Dir::get_target_files(&config) {
            Ok(dir)     => dir,
            Err(msg)    => panic!("{}", msg),
        };

        assert_eq!(3, test_dir.len());
    }
}

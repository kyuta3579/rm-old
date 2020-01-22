// rm-old
/* usage: rm-old -d [days] -riyv
    r:dir
    i:interactive
    y:assume yes
    v:verbose
    n:dry run
*/

use std::env;
use std::path::Path;
use std::time::SystemTime;
use std::fs;
use std::fs::File;

struct Dir{
    parent_path:    String,
    files_path:     Vec<String>,
}

impl Dir{
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
        if !self.files_path.is_empty() {
            println!("{}:",self.parent_path);
            for file in self.files_path.iter() {
                println!("    {}", file);
            }
            println!("\n");
        }
    }
}

struct Config {
    cur_path:       String,
    target_path:    Vec<String>,
    duration_days:  u64,
    change_days:    bool,
    do_intr:        bool,
    assume_yes:     bool,
    do_dir:         bool,
    verbose:        bool,
    dry_run:        bool,
}

impl Config {
    fn new () -> Config {
        Config {
            cur_path:       env::current_dir().unwrap().to_str().unwrap().to_string(),
            target_path:    Vec::new(),
            duration_days:  60,
            change_days:    false,
            do_intr:        false,
            assume_yes:     false,
            do_dir:         false,
            verbose:        false,
            dry_run:        false,
        }
    }

    fn print(&self) -> () {
        if self.verbose {
            for path in self.target_path.iter() {
                println!("target_path: {}", path);
            }
            println!("duration_days: {}", self.duration_days);
            if self.do_intr {
                println!("do_interact: yes");
            }
            if self.assume_yes {
                println!("assume_yes: yes");
            }
            if self.do_dir {
                println!("directory_target: yes");
            }
            if self.dry_run {
                println!("dry_run: yes");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let ret_config = match parse_config(&args) {
        Ok(config)    => config,
        Err(err_msg)      => {
            println!("{}\nusage: rm-old -d [days] -iryv", err_msg);
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

    for dir in target_files {
        dir.print();
    }

}

fn parse_config(args: &[String]) -> Result<Config, String> {
    let mut config = Config::new();
    let arg_iter = args.iter().skip(1);

    for arg in arg_iter {
        if !config.change_days && '-' == arg.as_str().chars().nth(0).unwrap() {
            match check_option(arg, &mut config) {
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
                match check_path(arg, &mut config){
                    Ok(_)  => {},
                    Err(err_msg)          => {return Err(err_msg);},
                }
            }
        }
    }

    if config.change_days {
        Err("rm-old -d: Input duration days after -d.".to_string())
    } else if config.target_path.is_empty(){
        config.target_path.push(config.cur_path.clone());
        Ok(config)
    } else {
        Ok(config)
    }

}

fn check_option(arg: &String, config: &mut Config) -> Result<(), String> {
    let c_iter = arg.as_str().chars().skip(1);
    for c in c_iter {
        match c {
            'd' => {
                config.change_days = true;
            },
            'r' => {
                config.do_dir = true;
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

fn check_path(arg: &String, config: &mut Config) -> Result<(), String> {
    let path = Path::new(arg);
    if path.exists() {
        if path.has_root() {
            config.target_path.push(arg.clone());
        } else {
            config.target_path.push(format!("{}/{}", config.cur_path, arg.clone()));
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
                target_dir.files_path.push(file_path.as_path().to_str().unwrap().to_string());
        } else if file_meta.is_dir() && config.do_dir {
            match get_files_in_dir(&(file_path.as_path().to_str().unwrap().to_string()), config, now)
            {
                Ok(mut files)       => {
                    target.append(&mut files);
                },
                Err(err_msg)    => return Err(err_msg),
            };
        }
    }
    target.push(target_dir);

    Ok(target)
}


fn execute_rm(target_files: Vec<String>, config: &Config) -> Result<(), String> {
    if target_files.is_empty() {
        return Err("Target files not exists!".to_string());
    }

    if config.assume_yes {
        for file_path in target_files.iter() {
            match fs::remove_file(Path::new(file_path)) {
                Ok(_)   => println!("removed: {}", file_path),
                Err(_)  => return Err("Fatal Error.".to_string()),
            }
        }
        return Ok(());
    }

    for file_path in target_files.iter() {
        println!("{}", file_path);
    }
    
    Ok(())
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_check_option() {
        let options = ['r', 'i', 'y', 'v', 'n'];
        let mut config = Config::new();

        for c in options.iter() {
            match c {
                'r' => {
                    assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_ok());
                    assert!(config.do_dir);
                },
                'i' => {
                    assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_ok());
                    assert!(config.do_intr);
                },
                'y' => {
                    assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_ok());
                    assert!(config.assume_yes);
                },
                'v' => {
                    assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_ok());
                    assert!(config.verbose);
                },
                'n' => {
                    assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_ok());
                    assert!(config.dry_run);
                },
                _   => assert!(check_option(&format!("-{}", c).to_string(), &mut config).is_err()),
            }
        }
    }

    #[test]
    fn test_parse_config() {
        let correct_args: Vec<Vec<String>> = vec![  //vec!["rm-old".to_string(), "/dev/zero/".to_string(), "-d".to_string(), "90".to_string(), "-riyvn".to_string()], 
                                                    vec!["rm-old".to_string(), "-d".to_string(), "90".to_string(), "-riyvn".to_string()],
                                                    vec!["rm-old".to_string(), "-d".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string(), "-riyvn".to_string()],
                                                    vec!["rm-old".to_string(), "-driyvn".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string()],
        ];

        let invalid_args: Vec<Vec<String>> = vec![  // vec!["rm-old".to_string(), "/dev/zero/not_exist/".to_string(), "90".to_string(), "-riyvnd".to_string()],
                                                    // After "-d" is not number.
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
}

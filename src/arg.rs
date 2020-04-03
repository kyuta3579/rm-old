use std::path::Path;

#[derive(Clone)]
pub struct Config {
    target_path:    Vec<String>,
    duration_days:  u64,
    change_days:    bool,
    do_intr:        bool,
    assume_yes:     bool,
    recursion:      bool,
    verbose:        bool,
    dry_run:        bool,
    remove_dir:     bool,
    remove_empty:   bool,
}

impl Config {
    pub fn new () -> Config {
        Config {
            target_path:    Vec::new(),
            duration_days:  60,
            change_days:    false,
            do_intr:        false,
            assume_yes:     false,
            recursion:      false,
            verbose:        false,
            dry_run:        false,
            remove_dir:     false,
            remove_empty:   false,
        }
    }

    pub fn print(&self) -> () {
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

    pub fn parse_config(args: &[String]) -> Result<Config, String> {
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

    pub fn get_duration_days(&self) -> u64 {
        self.duration_days
    }
    pub fn get_target_path(&self) -> Vec<String> {
        self.target_path.clone()
    }
    pub fn do_intr(&self) -> bool {
        self.do_intr
    }
    pub fn assume_yes(&self) -> bool {
        self.assume_yes
    }
    pub fn recursion(&self) -> bool {
        self.recursion
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
    pub fn remove_dir(&self) -> bool {
        self.remove_dir
    }
    pub fn remove_empty(&self) -> bool {
        self.remove_empty
    }
}

fn get_option(arg: &String, config: &mut Config) -> Result<(), String> {
    for c in arg.as_str()[1..].chars() {
        match c {
            '-' => {
                match analyze_long_option(arg, config) {
                    Ok(_) => break,
                    Err(err_msg) => return Err(format!("{}", err_msg)),
                }
            },
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

fn analyze_long_option(arg: &String, config: &mut Config) -> Result<(), String>{
    if arg == "--remove-dir" {
        config.remove_dir = true;
        Ok(())
    } else if arg == "--remove-empty"{
        config.remove_empty = true;
        Ok(())
    } else {
        Err(format!("rm-old: illegal option: {}", arg))
    }
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

fn show_help() -> String {
    "rm-old: remove the old files in dir.
    -r              : recursion. target dir in dir.
    -i              : ask each file when remove.
    -y              : assume yes.
    -d [days]       : specify duration of day.(default is 60 days)
    -v              : verbose.
    -n              : dry run. not a remove, only show log.
    --remove-dir    : remove directory.
    --remove-empty:
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
        let correct_args: Vec<Vec<String>> = vec![  vec!["rm-old".to_string(), "-d".to_string(), "90".to_string(), "-riyvn".to_string(), "--remove-dir".to_string(), "--remove-empty".to_string()],
                                                    vec!["rm-old".to_string(), "-d".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string(), "-riyvn".to_string()],
                                                    vec!["rm-old".to_string(), "-driyvn".to_string(), "90".to_string()],
                                                    vec!["rm-old".to_string(), "--remove-dir".to_string()],
                                                    vec!["rm-old".to_string(), "--remove-empty".to_string()],
                                                    vec!["rm-old".to_string(), "--remove-empty".to_string()],
        ];

        let invalid_args: Vec<Vec<String>> = vec![  // After "-d" is not number.
                                                    vec!["rm-old".to_string(), "-d".to_string(), "-riyvn".to_string()],
                                                    // not path.
                                                    vec!["rm-old".to_string(), "jifsl.?s_sdfe".to_string()],
                                                    // After "-d" not exist.
                                                    vec!["rm-old".to_string(), "-driyvn".to_string()],
                                                    // not supported.
                                                    vec!["rm-old".to_string(), "--get-list".to_string()],
        ];

        for arg in correct_args.iter() {
            assert!(Config::parse_config(&arg).is_ok());
        }
        for arg in invalid_args.iter() {
            assert!(Config::parse_config(&arg).is_err());
        }
    }
}
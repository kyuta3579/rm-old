// rm-old
/* usage: rm-old -d [days] -riyv
    r:dir
    i:interactive
    y:assume yes
    v:verbose
*/

use std::env;
use std::io;
use std::fs;

struct Config {
    duration_days:  u32,
    do_intr:        bool,
    assume_yes:     bool,
    do_dir:         bool,
    verbose:        bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let ret_config = match parse_config(&args) {
        Ok(config)    => config,
        Err(str)      => {
            panic!("{:?}: usage: rm-old -d [days] -iryv", str);
        },
    };
    
    println!("days = {}",ret_config.duration_days);
}

fn parse_config(args: &[String]) -> Result<Config, String> {
    let mut config = Config {
        duration_days:  60,
        do_intr:        false,
        assume_yes:     false,
        do_dir:         false,
        verbose:        false,
    };

    let mut is_days = false;
    let mut is_first = true;
    for arg in args.iter() {
        if !is_first {
            if '-' == arg.as_str().chars().nth(0).unwrap() {
                for c in arg.as_str().chars() {
                    match c {
                        '-' =>{},
                        'd' => {
                            is_days = true;
                            println!("-d");
                        },
                        'r' => {
                            config.do_dir = true;
                            println!("-r");
                        },
                        'i' => {
                            config.do_intr = true;
                            println!("-i");
                        },
                        'y' => {
                            config.assume_yes = true;
                            println!("-y");
                        },
                        'v' => {
                            config.verbose = true;
                            println!("-v");
                        },
                        _ => {
                            println!("{}",c);
                            return Err(format!("rm-old: illegal option:{}", c).to_string());
                        },
                    }
                }
            } else {
                if is_days {
                    let days = arg.parse::<u32>();
                    match days {
                        Ok(num)   => {
                            config.duration_days = num;
                        },
                        Err(_)    => {
                            return Err(format!("rm-old -d: illegal value: {}", arg));
                        },
                    }
                } else {
                    return Err(format!("rm-old: illegal option: {}", arg).to_string());
                }
            }
        } else {
            is_first = false;
        }
    }

    Ok(config)
}


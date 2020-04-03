use crate::arg::Config;

use std::time::SystemTime;
use std::fs::{self, File};
use std::sync::mpsc;
use std::thread;



pub struct Dir{
    pub parent_path:    String,
    pub files_path:     Vec<String>,
}

impl Dir {
    pub fn new(path: &String) -> Dir {
        Dir {
            parent_path:    path.clone(),
            files_path:     Vec::new(),
        }
    }

    pub fn get_target_files(config: &Config) -> Result<Vec<Dir>, String> {
        let now_sys_time             = SystemTime::now();
        let mut targets: Vec<Dir>    = Vec::new();

        if !config.remove_dir(){
            for path in config.get_target_path().iter() {
                let mut t = match get_files_in_dir(path, config.clone(), now_sys_time) {
                    Ok(files)       => files,
                    Err(err_msg)    => return Err(err_msg),
                };
                targets.append(&mut t);
            }
        } else {
            for path in config.get_target_path().iter() {
                let t = match get_dirs_in_dir(path, &config, now_sys_time) {
                    Ok(dirs)       => dirs,
                    Err(err_msg)    => return Err(err_msg),
                };
                targets.push(t);
            }
        }
        println!("{:?}", SystemTime::now().duration_since(now_sys_time));
        Ok(targets)
    }

    pub fn print(&self) -> () {
        println!("{}/:",self.parent_path);
        for file in self.files_path.iter().rev() {
            println!("    {}", file);
        }
        println!("");
    }

    pub fn get_amount_files(&self) -> u64 {
        self.files_path.len() as u64
    }

    pub fn get_parent_path(&self) -> &String {
        &self.parent_path
    }

    pub fn get_files(&self) -> &Vec<String> {
        &self.files_path
    }
}

fn get_files_in_dir(path: &String, config: Config, now: SystemTime) -> Result<Vec<Dir>, String> {
    let mut target: Vec<Dir> = Vec::new();
    let mut search_dir: Dir = Dir::new(path);
    let mut thread_pool: Vec<ThreadNode> = Vec::new();

    let files = match fs::read_dir(path) {
        Err(why)    => {
            return Err(format!("Can not open dir: {:?}", why.kind()));
        },
        Ok(paths)   => paths,
    };

    for f in files {
        let file_path           = f.unwrap().path();
        let file_meta           = File::open(&file_path).unwrap().metadata().unwrap();

        let duration_time_by_ac = match now.duration_since(file_meta.accessed().unwrap()){
            Ok(duration)    => duration.as_secs(),
            Err(e)          => {
                return Err(format!("Clock may have gone backwards: {:?}",e.duration()));
            },
        };

        if file_meta.is_file() && (config.get_duration_days() * 86400 < duration_time_by_ac) {

            search_dir.files_path.push(
                file_path.as_path().file_name().unwrap().to_str().unwrap().to_string());

        } else if file_meta.is_dir() && config.recursion() {
            let (child_sender, child_reciever) = mpsc::channel::<Result<Vec<Dir>, String>>();
            let tmp_config = config.clone();

            let handle = thread::spawn(move || {
                let res = get_files_in_dir(
                    &(file_path.as_path().to_str().unwrap().to_string()), tmp_config, now);
                child_sender.send(res).unwrap();
            });
            thread_pool.push(ThreadNode::new(handle, child_reciever));
        }
    }

    if !search_dir.files_path.is_empty() {
        target.push(search_dir);
    }

    if !thread_pool.is_empty() {
        for node in thread_pool {
            match node.listen(){
                Ok(mut res) => {
                    target.append(&mut res);
                    for d in res {
                        d.print();
                    }
                },
                Err(err_msg) => println!("{}", err_msg),
            }
            node.handle.join().unwrap();
        }
    }

    Ok(target)
}

fn get_dirs_in_dir(path: &String, config: &Config, now: SystemTime) -> Result<Dir, String>{
    let files = match fs::read_dir(path) {
        Err(why)    => {
            return Err(format!("Can not open dir: {:?}", why.kind()));
        },
        Ok(paths)   => paths,
    };

    let mut target_dir = Dir::new(path);

    for f in files {
        let file_path           = f.unwrap().path();
        let file_meta           = File::open(&file_path).unwrap().metadata().unwrap();
        let duration_time_by_ac = match now.duration_since(file_meta.accessed().unwrap()){
            Ok(duration)    => duration.as_secs(),
            Err(e)          => {
                return Err(format!("Clock may have gone backwards: {:?}",e.duration()));
            },
        };

        if file_meta.is_dir() && (config.get_duration_days() * 86400 < duration_time_by_ac) {
            target_dir.files_path.push(
                file_path.as_path().file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    if !target_dir.files_path.is_empty() {
        target_dir.parent_path = path.clone();
        return Ok(target_dir);
    } else {
        return Err(format!("Target directories not exists!"));
    }
}

struct ThreadNode {
    handle: thread::JoinHandle<()>,
    listener: mpsc::Receiver<Result<Vec<Dir>, String>>,
}

impl ThreadNode {
    fn new(handle: thread::JoinHandle<()>, listener: mpsc::Receiver<Result<Vec<Dir>, String>>) -> ThreadNode {
        ThreadNode {
            handle,
            listener,
        }
    }

    fn listen(&self) -> Result<Vec<Dir>, String> {
        match self.listener.recv() {
            Ok(res)       => {
                match res {
                    Ok(dir)    => {
                        return Ok(dir);
                    },
                    Err(err_msg)    => return Err(err_msg),
                }
            },
            Err(_)        => {
                return Err("Can not recieve Message!".to_string());
            }
        };
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_get_files() {
        let config = Config::parse_config(&vec!["rm-old".to_string(), "test_dir".to_string(), "-d".to_string(), "0".to_string()]).unwrap();

        let mut test_dir = match Dir::get_target_files(&config) {
            Ok(dir)     => dir,
            Err(msg)    => panic!("{}", msg),
        };
        assert_eq!(1, test_dir.len());

        let config = Config::parse_config(&vec!["rm-old".to_string(), "test_dir".to_string(), "-d".to_string(), "0".to_string(), "-r".to_string()]).unwrap();

        test_dir = match Dir::get_target_files(&config) {
            Ok(dir)     => dir,
            Err(msg)    => panic!("{}", msg),
        };

        assert_eq!(3, test_dir.len());

        let config = Config::parse_config(&vec!["rm-old".to_string(), "test_dir".to_string(), "-d".to_string(), "0".to_string(), "--remove-dir".to_string()]).unwrap();

        test_dir = match Dir::get_target_files(&config) {
            Ok(dir)     => dir,
            Err(msg)    => panic!("{}", msg),
        };

        assert_eq!(1, test_dir.len());
    }
}
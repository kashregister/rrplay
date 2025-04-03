use crate::term_utils::*;
use std::path::Path;
use std::process::exit;
pub struct ConfigHandler {
    pub sources: Vec<(String, bool)>,
}

impl ConfigHandler {
    pub fn init() -> ConfigHandler {
        let r = Vec::new();
        return ConfigHandler { sources: r };
    }
    pub fn startup(&mut self) {
        if self.config_check_file_exist() {
            self.config_validate();
        } else {
            self.config_create();
        }
    }
    pub fn config_check_file_exist(&mut self) -> bool {
        if let Some(cfg_dir) = dirs::config_dir() {
            let config_file = cfg_dir.join("rrplay").join("config");
            if !config_file.is_file() {
                return false;
            } else {
                return true;
            }
        } else {
            println!("failed fetching config dir");
            return false;
        }
    }
    pub fn config_create(&mut self) {
        println!("No config found, creating under .config/rrplay/config");
        t_mv_sol();
        println!("Enter the full path of your library for example: /home/kr24/Music");
        t_mv_sol();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Err");
        t_flush();
        let input = input.trim();
        // println!("{input}");
        if Path::new(&input).exists() {
            println!("Path valid, creating config file...");
            t_mv_sol();
            if let Some(cfg_dir) = dirs::config_dir() {
                let mut config_file = cfg_dir.join("rrplay");
                std::fs::create_dir_all(config_file.clone()).unwrap();

                println!("writing to {}", config_file.to_str().unwrap());

                config_file = cfg_dir.join("rrplay/config");
                std::fs::write(config_file, input).unwrap();
                println!("File created!");

                t_mv_sol();
                let path = input.to_string().trim().to_string();
                self.sources.push((path, true));
            } else {
                println!("Path invalid, aborting...");
                exit(1);
            }
        } else {
            println!("Path does not exist");
            exit(1);
        }
    }
    pub fn config_validate(&mut self) {
        if let Some(cfg_dir) = dirs::config_dir() {
            let config_file = cfg_dir.join("rrplay").join("config");
            let file_contents = std::fs::read_to_string(config_file).unwrap();
            let paths = file_contents.split("\n");

            for path in paths {
                let ap = path.trim().to_string();

                if Path::new(&ap).exists() {
                    self.sources.push((ap, true));
                } else {
                    self.sources.push((ap, false));
                }
            }
        }
    }
}

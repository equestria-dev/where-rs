use std::{env, fs};
use std::path::PathBuf;
use serde::Deserialize;
use crate::args::Args;

const TIMEOUT: u64 = 2000;
const MAX_SEND_RETRIES: usize = 3;
const CONFIG_FILENAME: &str = "where.toml";

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    pub global: GlobalConfig,
    pub server: Vec<Server>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GlobalConfig {
    pub timeout: u64,
    pub max_retries: usize,
    pub include_inactive: bool,
    pub port: u16,
    pub source: String
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub endpoint: String,
    pub label: Option<String>,
    pub timeout: Option<u64>,
    pub max_retries: Option<usize>,
    pub failsafe: Option<bool>
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            timeout: TIMEOUT,
            max_retries: MAX_SEND_RETRIES,
            include_inactive: true,
            port: 15,
            source: "Local".to_string()
        }
    }
}

impl Config {
    fn get_config_locations() -> Vec<PathBuf> {
        vec![
            {
                let mut path = PathBuf::new();

                if let Ok(home) = env::var("XDG_CONFIG_HOME") {
                    path.push(home);
                } else if let Ok(home) = env::var("HOME") {
                    path.push(home);
                    path.push(".config");
                } else {
                    path.push("/");
                }

                path.push(CONFIG_FILENAME);
                path
            },
            {
                let mut path = PathBuf::new();

                path.push("/etc");
                path.push(CONFIG_FILENAME);
                path
            }
        ]
    }

    pub fn build(args: Args) -> Self {
        let config: Option<Config> = Self::get_config_locations()
            .iter()
            .flat_map(|path| fs::read_to_string(path).ok())
            .map(|str| toml::from_str(&str).unwrap_or_else(|e| {
                eprintln!("where: Failed to parse configuration file: {e}");
                std::process::exit(1);
            }))
            .next();

        if args.generate_config {
            let default_config = include_str!("../default_config.toml");

            let mut saved_path: Option<&PathBuf> = None;
            let mut save_locations = Self::get_config_locations();
            save_locations.reverse();

            let res: Option<()> = save_locations
                .iter()
                .flat_map(|path| {
                    let res = fs::write(path, default_config).ok();

                    if res.is_some() {
                        saved_path = Some(path);
                    }

                    res
                })
                .next();

            if res.is_some() {
                println!("where: Generated default configuration file at {}. Please edit it and run 'where' again.", saved_path.unwrap().to_str().unwrap());
                std::process::exit(0);
            } else {
                let save_locations_strings: Vec<String> = save_locations
                    .into_iter()
                    .map(|path| path.to_str().unwrap().to_string())
                    .collect();
                println!("where: Failed to generate the default configuration file, tried: {}", save_locations_strings.join(", "));
                std::process::exit(1);
            }
        } else {
            let locations_strings: Vec<String> = Self::get_config_locations()
                .into_iter()
                .map(|path| path.to_str().unwrap().to_string())
                .collect();

            config.unwrap_or_else(|| {
                eprintln!("where: Valid configuration file found nowhere, tried: {}\nPass -c to generate a default config file.", locations_strings.join(", "));
                std::process::exit(1);
            })
        }
    }
}

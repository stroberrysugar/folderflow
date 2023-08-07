use std::{collections::HashSet, io::ErrorKind, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub listen_address: SocketAddr,
    pub root_folder_directory: PathBuf,
    pub temp_zip_directory: PathBuf,
    pub max_upload_size_in_bytes: usize,
    #[serde(rename = "script_config")]
    pub scripts: Vec<ScriptConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScriptConfig {
    pub id: u16,
    pub friendly_name: String,
    pub path_to_script: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:8082".parse().unwrap(),
            root_folder_directory: std::env::current_dir().unwrap().join("uploaded_folders"),
            temp_zip_directory: "/tmp".parse().unwrap(),
            max_upload_size_in_bytes: 4294967296,
            scripts: vec![ScriptConfig {
                id: 0,
                friendly_name: "script 1".to_string(),
                path_to_script: "/home/user/testscript.sh".parse().unwrap(),
            }],
        }
    }
}

pub fn get_config() -> Config {
    let config = match std::fs::read_to_string("config.toml") {
        Ok(n) => match toml::from_str::<Config>(&n) {
            Ok(n) => n,
            Err(e) => panic!("Failed to parse config.toml: {:?}", e),
        },
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                let default = Config::default();

                std::fs::write("config.toml", toml::to_string_pretty(&default).unwrap())
                    .expect("Failed to write default config.toml file");

                default
            }
            _ => panic!("Failed to read config.toml: {:?}", e),
        },
    };

    let mut script_ids = HashSet::new();

    for script in config.scripts.iter() {
        if !script_ids.insert(script.id) {
            panic!("Duplicate script ID `{}` in config.toml", script.id);
        }
    }

    config
}

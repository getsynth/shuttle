use std::{fs, path::PathBuf};

pub fn get_api_key() -> String {
    if let Ok(s) = std::env::var("SHUTTLE_API_KEY") {
        return s;
    }

    let data = fs::read_to_string(config_path()).expect("shuttle config file to exist");
    let toml: toml::Value = toml::from_str(&data).expect("to parse shuttle config file");

    toml["api_key"]
        .as_str()
        .expect("api key to be a string")
        .to_string()
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("system to have a config path")
        .join("shuttle")
        .join("config.toml")
}

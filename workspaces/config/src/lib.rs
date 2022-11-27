use std::fs;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub hooks: Hooks
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub listen_socket: String,
    pub wm_socket: String,
}

#[derive(Deserialize, Clone)]
pub struct Hooks {
    pub exec_ignore: HashSet<String>,
    pub hooks: Vec<Hook>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Hook {
    pub event: String,  // TODO: see if i can make this an enum
    pub exec: String,
}

pub type GenericRes = (u8, Option<String>);
pub type OptGenRes = Option<GenericRes>;

const CONFIG_FILE: &str = "~/.config/desktop-automater/config.toml";

pub fn get_configs() -> Result<Config, std::io::Error> {
    let fname = shellexpand::tilde(CONFIG_FILE).to_string();
    let toml_file: String = fs::read_to_string(fname)?;
    Ok(toml::de::from_str(&toml_file)?)
}
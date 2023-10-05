use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

pub const PORT_PIPE: &str = "auto-desk.ports";

#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub hooks: Hooks,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub listen_socket: String,
    pub wm_socket: String,
}

#[derive(Deserialize, Clone)]
pub struct Hooks {
    pub exec_ignore: HashSet<String>,
    pub ignore_web: bool,
    pub listen: Option<bool>,
    pub hooks: Vec<Hook>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Hook {
    pub event: String, // TODO: see if i can make this an enum
    pub exec: String,
}

pub type GenericRes = (u8, Option<String>);
pub type OptGenRes = Option<GenericRes>;

const CONFIG_FILE: &str = "~/.config/auto-desk/config.toml";

pub fn get_configs() -> Result<Config, std::io::Error> {
    let fname = shellexpand::tilde(CONFIG_FILE).to_string();
    let toml_file: String = fs::read_to_string(fname)?;
    Ok(toml::de::from_str(&toml_file)?)
}

/// returns the runtime dir to use for named pipes and stuff.  
pub fn get_pipe_d() -> String {
    // let file_name = "ports-data.pipe";
    // match BaseDirectories::with_prefix("auto-desk") {
    //     Ok(run_dir) => {
    //         match run_dir.find_runtime_file(&file_name) {
    //             Some(path) => Ok(path.to_string_lossy().to_string()),
    //             None => {
    //                 // println!(");
    //                 let error_mesg = "[ERROR] Couldn't find the auto-desk ports-data.pipe file.";
    //                 println!("{error_mesg}");
    //                 Err(String::from(error_mesg))
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         let error_mesg = "[ERROR] Couldn't find the auto-desk run dir. got error: \n{e}";
    //         println!("{error_mesg}");
    //         Err(String::from(error_mesg))
    //     }
    // }

    // match env::var("XDG_RUNTIME_DIR") {
    //     Ok(dir) => {
    //         format!("{dir}/auto-desk")
    //         // Ok(format!("{dir}/auto-desk"))
    //     }
    //     Err(e) => {
    //         let uid = users::get_current_uid();
    //         format!("/run/user/{uid}/auto-desk")
    //         // Err(format!("{e}")),
    //     }
    // }
    String::from("/tmp")
}

pub fn get_pipe_f() -> String {
    format!("{}/{PORT_PIPE}", get_pipe_d())
}

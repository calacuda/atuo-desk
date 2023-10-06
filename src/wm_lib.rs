#![deny(clippy::all)]
use log::error;
use serde::{Deserialize, Serialize};
use serde_yaml;
use shellexpand;
use std::path::Path;
use std::{collections::HashMap, fs::read_to_string};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Program {
    pub name: String,
    pub state: Option<String>,
    pub wm_class: Option<String>,
    pub args: Option<Vec<String>>,
    pub delay: Option<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Conf {
    pub desktops: Vec<DesktopLayout>,
    pub workspaces: Option<HashMap<i32, i32>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct DesktopLayout {
    pub desktop: String,
    pub asyncro: Option<bool>,
    pub programs: Vec<Program>,
    pub clear: Option<bool>,
}

pub fn get_layout(fname: &str) -> Result<Conf, u8> {
    let file_path = match get_layout_file(fname) {
        Ok(path) => path,
        Err(_) => {
            error!("can't load layout stored in \"{fname}\", file doesn't exsist.");
            return Err(4);
        }
    };

    let layout_file = match read_to_string(&file_path) {
        Ok(data) => data,
        Err(_) => {
            error!("could not layout file \"{file_path}\"");
            return Err(4);
        }
    };

    match serde_yaml::from_str(&layout_file) {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("could not parse yaml layout file {fname}. error: \"{e}\"");
            Err(4)
        }
    }
}

fn get_layout_file(file_name: &str) -> Result<String, ()> {
    // let shellexpand::tilde(
    //     &if file_name.ends_with(".layout") || file_name.ends_with(".yml") {
    //         format!("~/.config/auto-desk/layouts/{}", file_name)
    //     } else {
    //         format!("~/.config/auto-desk/layouts/{}.layout", file_name)
    //     },
    // )
    // .to_string();
    #[cfg(feature = "testing")]
    {
        if Path::new(file_name).exists() {
            return Ok(Path::new(file_name).to_str().unwrap().to_string());
        }
    }

    // TODO: pull path from config file
    let mut layout_dir = shellexpand::tilde("~/.config/auto-desk/layouts/").to_string();

    if shellexpand::tilde(&file_name)
        .to_string()
        .starts_with(&layout_dir)
        && Path::new(file_name).exists()
    {
        return Ok(shellexpand::tilde(file_name).to_string());
    }

    // TODO: pull path from config file
    layout_dir =
        shellexpand::tilde(&format!("~/.config/auto-desk/layouts/{}", file_name)).to_string();

    let f_types = ["", ".yml", ".yaml", ".layout"];

    for f_type in f_types {
        let p = Path::new(&format!("{}{}", layout_dir, f_type)).to_owned();
        if p.exists() {
            return Ok(p.to_str().unwrap().to_string());
        }
    }
    Err(())
}

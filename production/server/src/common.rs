// use std::collections::HashMap;
// use std::error::Error;
// use std::io::prelude::*;
use rdev::{simulate, EventType, Key, SimulateError};
use shellexpand;
use std::path::Path;
use std::process::Command;
use std::{thread, time};

pub mod backlight;
pub mod media;
pub mod power;
pub mod xrandr;

/*
 * TODOs:
 * - make the open_program routine like that of open_on_desktop form the BSPWM mod
 *
 */

pub fn get_layout_file(file_name: &str) -> Result<String, ()> {
    // let shellexpand::tilde(
    //     &if file_name.ends_with(".layout") || file_name.ends_with(".yml") {
    //         format!("~/.config/desktop-automater/layouts/{}", file_name)
    //     } else {
    //         format!("~/.config/desktop-automater/layouts/{}.layout", file_name)
    //     },
    // )
    // .to_string();
    #[cfg(feature = "testing")]
    {
        if Path::new(file_name).exists() {
            return Ok(Path::new(file_name).to_str().unwrap().to_string());
        }
    }

    let mut layout_dir = shellexpand::tilde("~/.config/desktop-automater/layouts/").to_string();

    if shellexpand::tilde(&file_name)
        .to_string()
        .starts_with(&layout_dir)
        && Path::new(file_name).exists()
    {
        return Ok(shellexpand::tilde(file_name).to_string());
    }

    layout_dir = shellexpand::tilde(&format!(
        "~/.config/desktop-automater/layouts/{}",
        file_name
    ))
    .to_string();

    let f_types = ["", ".yml", ".yaml", ".layout"];

    for f_type in f_types {
        let p = Path::new(&format!("{}{}", layout_dir, f_type)).to_owned();
        if p.exists() {
            return Ok(p.to_str().unwrap().to_string());
        }
    }
    return Err(());
}

pub fn open_program(program: &str) -> u8 {
    println!("[LOG] running: {}", program);
    let _process = Command::new(program)
        .output()
        .expect("failed to execute process");
    println!("[LOG] ran {}", program);
    return 0;
}

fn send_key_stroke(event_type: &EventType) -> u8 {
    let delay = time::Duration::from_millis(20);
    let res = match simulate(event_type) {
        Ok(()) => 0,
        Err(SimulateError) => {
            match event_type {
                EventType::KeyPress(Key::PrintScreen) => {
                    println!("[ERROR] print screen key not pressed")
                }
                EventType::KeyRelease(Key::PrintScreen) => {
                    println!("[ERROR] print screen key not released")
                }
                EventType::KeyPress(key) => println!("[ERROR {:?} key not pressed", key),
                EventType::KeyRelease(key) => println!("[ERROR {:?} key not released", key),
                _ => {} // this will happend when funciton is used for somthing unintended.
            }
            4
        }
    };
    // Let ths OS catchup (at least MacOS)
    thread::sleep(delay);
    return res;
}

pub fn screen_shot() -> u8 {
    let press = send_key_stroke(&EventType::KeyPress(Key::PrintScreen));
    let release = send_key_stroke(&EventType::KeyRelease(Key::PrintScreen));

    return if press > 0 {
        press
    } else if release > 0 {
        release
    } else {
        0
    };
}

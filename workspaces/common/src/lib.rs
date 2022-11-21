// use std::collections::HashMap;
// use std::error::Error;
// use std::io::prelude::*;
use rdev::{simulate, EventType, Key, SimulateError};
use std::process::{Command, Stdio};
use std::{thread, time};

pub mod backlight;
pub mod media;
pub mod power;
pub mod xrandr;

pub fn open_program(program: &str) -> u8 {
    println!("[LOG] running: {}", program);
    let process = if program.ends_with(".desktop") {
        Command::new("gtk-launch")
            .arg(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    println!("[LOG] ran {}", program);
    return match process {
        Ok(_) => {
            println!("[LOG] program {} launched", program);
            0
        }
        Err(e) => {
            println!("[ERROR] program {} could not be launched: {}", program, e);
            4
        }
    };
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

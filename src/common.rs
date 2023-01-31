use rdev::{simulate, EventType, Key, SimulateError};
use std::process::{Command, Stdio};
use std::{thread, time};
use crate::config::OptGenRes;

mod backlight;
mod media;
mod power;
mod xrandr;

pub fn open_program(program: &str) -> u8 {
    // TODO: make the programs keep running after desktop-automater stops or gets killed.
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
            .arg(format!("coproc {program}; disown; exit"))  
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    println!("[LOG] ran '{}'", program);
    match process {
        Ok(_) => {
            println!("[LOG] program '{}' launched", program);
            0
        }
        Err(e) => {
            println!("[ERROR] program '{}' could not be launched: '{}'", program, e);
            4
        }
    }
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
    res
}

fn screen_shot() -> u8 {
    let press = send_key_stroke(&EventType::KeyPress(Key::PrintScreen));
    let release = send_key_stroke(&EventType::KeyRelease(Key::PrintScreen));

    if press > 0 {
        press
    } else if release > 0 {
        release
    } else {
        0
    }
}

pub async fn common_switch(cmd: &str, args: &str) -> OptGenRes{
    match cmd {
        "open-here" => Some((open_program(args), None)),
        "screen-shot" => Some((screen_shot(), None)),
        "inc-bl" => Some((backlight::inc_bright(args), None)),
        "dec-bl" => Some((backlight::dec_bright(args), None)),
        "add-monitor" => Some((xrandr::add_monitor(args), None)),
        _ => None,
    }
}

#[cfg(feature = "systemctl")]
pub async fn sysctl_switch(cmd: &str) -> OptGenRes {
    match cmd {
        "poweroff" => Some((power::power_off(), None)),
        "hibernate" => Some((power::hibernate(), None)),
        "reboot" => Some((power::reboot(), None)),
        "sleep" | "suspend" => Some((power::sleep(), None)),
        "lock" => Some((power::lock(), None)),
        "logout" => Some((power::logout(), None)),
        _ => None,
    }
}

#[cfg(feature = "media")]
pub async fn media_switch(cmd: &str, args: &str) -> OptGenRes {
    match cmd {
        "vol-up" => Some((media::volume_up(args), None)),
        "vol-down" => Some((media::volume_down(args), None)),
        "mute" => Some((media::mute(), None)),
        "play/pause" => Some((media::play_pause(), None)),
        "play-track" => Some((media::play(), None)),
        "pause-track" => Some((media::pause(), None)),
        "stop-track" => Some((media::stop(), None)),
        "next-track" => Some((media::next_track(), None)),
        "last-track" => Some((media::last_track(), None)),
        _ => None,
    }
}
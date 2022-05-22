use configparser::ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
// use unix_socket::Incoming;

mod bspwm;
// mod common;
mod common;
// mod power;

fn load_config(
    config_file: &str,
) -> Result<HashMap<String, HashMap<String, Option<String>>>, Box<dyn Error>> {
    let mut config = Ini::new();
    let map = config.load(shellexpand::tilde(config_file).to_string())?;
    return Ok(map);
}

fn get_servers(config_file: &str) -> HashMap<String, Option<String>> {
    let configs: HashMap<String, HashMap<String, Option<String>>> = match load_config(config_file) {
        Ok(data) => data,
        Err(e) => panic!(
            "got error : {:?}\nwrite and ensure_dir and ensure_file and put them here",
            e
        ),
    };

    // println!("{:#?}", configs);

    return match configs.get("server") {
        Some(data) => data.to_owned(),
        None => panic!("config file has not server configurations. exiting"),
    };
}

fn write_shutdown(stream: &mut UnixStream, res: u8) {
    if res > 0 {
        stream
            .write_all(&format!("{}{}", res as char, res).as_bytes())
            .unwrap();
    } else {
        stream
            .write_all(&format!("{} done", res).as_bytes())
            .unwrap();
    }
    stream.shutdown(std::net::Shutdown::Write);
}

fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    stream.read_to_string(&mut command).unwrap();
    stream.shutdown(std::net::Shutdown::Read);
    return command;
}

fn switch_board(command: String, spath: &str) -> u8 {
    let (cmd, args) = match command.split_once(" ") {
        Some(cmd_args) => cmd_args.to_owned(),
        None => (command.as_str(), ""),
    };
    return match cmd {
        "open-here" => common::open_program(args),
        "screen-shot" => common::screen_shot(),
        "focus-on" => bspwm::focus_on(spath, args),
        "move-to" => bspwm::move_to(spath, args),
        "close-focused" => bspwm::close_focused(spath),
        "open-at" => bspwm::open_on_desktop(spath, args),
        "poweroff" => common::power::power_off(),
        "hibernate" => common::power::hibernate(),
        "reboot" => common::power::reboot(),
        "sleep" | "suspend" => common::power::sleep(),
        "lock" => common::power::lock(),
        "logout" => common::power::logout(),
        "vol-up" => common::media::volume_up(args),
        "vol-down" => common::media::volume_down(args),
        "mute" => common::media::mute(),
        "play/pause" => common::media::play_pause(),
        "play-track" => common::media::play(),
        "pause-track" => common::media::pause(),
        "stop-track" => common::media::stop(),
        "next-track" => common::media::next_track(),
        "last-track" => common::media::last_track(),
        "inc-bl" => common::backlight::inc_bright(args),
        "dec-bl" => common::backlight::dec_bright(args),
        "add-monitor" => common::xrandr::add_monitor(args),
        _ => 1,
    };
}

fn handle_client(mut stream: UnixStream, spath: &str) {
    let command = read_command(&mut stream);
    // println!("{}", command);

    // handle comand here
    // let res: u8 = 0;
    let res: u8 = switch_board(command, spath);
    write_shutdown(&mut stream, res);
    drop(stream)
}

fn recv_loop(progr: &str, bspwm: String) -> std::io::Result<()> {
    println!("recv_loop");
    let listener = UnixListener::bind(progr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let tmp_bspwm = bspwm.clone();
                thread::spawn(move || handle_client(stream, &tmp_bspwm));
            }
            Err(err) => {
                println!("{:#?}", err);
                /* connection failed */
                break;
            }
        }
    }

    println!("killing listener");
    drop(listener);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let configs: HashMap<String, Option<String>> =
        get_servers(&"~/.config/desktop-automater/config.ini");
    let (prog_so, bspwm_so) = match (configs.get("prog-so"), configs.get("bspwm-so")) {
        (Some(Some(p)), Some(Some(b))) => (p.to_owned(), b.to_owned()),
        (Some(Some(p)), None) => {
            println!("bspwm socket location not specified, using default.");
            (p.to_owned(), "/tmp/bspwm_0_0-socket".to_string())
        }
        (None, Some(Some(b))) => {
            println!("program socket location not specified, using default.");
            ("/tmp/desktop-automater".to_string(), b.to_owned())
        }
        (None, None) => {
            println!("neither socket location were specified, using defaults.");
            (
                "/tmp/desktop-automater".to_string(),
                "/tmp/bspwm_0_0-socket".to_string(),
            )
        }
        _ => panic!("configs corrupted. the configuration options needs to be set or not present. manual editing is adviced")
    };
    // println!("{:#?}", configs);
    // println!("progr {}\nbspwm {}", prog_so, bspwm_so);
    if std::fs::metadata(&prog_so).is_ok() {
        std::fs::remove_file(&prog_so).expect(&format!(
            "could not delete previous socket at {:?}",
            &prog_so
        ));
    }
    match recv_loop(&prog_so, bspwm_so) {
        Ok(_) => {}
        Err(e) => println!("[ERROR] {}", e),
    }
    // println!("Goodbye!");
    Ok(())
}

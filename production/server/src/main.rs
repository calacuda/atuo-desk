use configparser::ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
// use std::io::prelude::*;
use std::fs::read_to_string;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::{thread, time};
// use yaml_rust::{YamlLoader, Yaml};
// use unix_socket::Incoming;
use serde::{Deserialize, Serialize};
use serde_yaml;

mod bspwm;
mod common;
// mod free_desktop;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Program {
    name: String,
    state: Option<String>,
    delay: Option<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DesktopLayout {
    desktop: u8,
    programs: Vec<Program>,
}

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
        Err(e) => panic!("got error : {:?}\n", e),
    };

    // println!("{:#?}", configs);

    return match configs.get("server") {
        Some(data) => data.to_owned(),
        None => panic!("config file has not server configurations. exiting"),
    };
}

fn load_from_layout(layout_file: String, spath: &str) -> u8 {
    for cmd in layout_file.lines() {
        let err_code = switch_board(cmd.to_string(), spath);
        if err_code > 0 {
            return err_code;
        }
    }
    return 0;
}

fn set_up_desktop(
    desktop_name: &str,
    programs: &Vec<Program>,
    spath: &str,
    all_rules: &mut Vec<String>,
) -> u8 {
    for program in programs {
        // program = program.to_owned();
        let rules = [
            format!(
                "{}:{} desktop={}",
                program.name[0..1].to_uppercase() + &program.name[1..],
                program.name,
                desktop_name
            ),
            format!("{} desktop={}", program.name, desktop_name),
            format!(
                "{}:{} desktop={}",
                program.name[0..1].to_uppercase() + &program.name[1..],
                program.name[0..1].to_uppercase() + &program.name[1..],
                desktop_name
            ),
        ];

        for rule in &rules {
            all_rules.push(rule.to_owned());
            if bspwm::send(spath, &format!("rule -a {} --one-shot", &rule)) > 0 {
                return 3;
            }
        }

        let error_code =
            bspwm::open_on_desktop(spath, &format!("{} {}", &program.name, desktop_name));
        if error_code > 0 {
            return error_code;
        }
        match program.delay {
            Some(times) => {
                let t = time::Duration::from_millis(100);
                for _ in 0..times {
                    thread::sleep(t);
                }
            }
            None => {}
        }
    }
    return 0;
}

fn load_from_yaml(layout_file: String, spath: &str, fname: &str) -> u8 {
    let layouts: Vec<DesktopLayout> = match serde_yaml::from_str(&layout_file) {
        Ok(data) => data,
        Err(e) => {
            println!("[ERROR] could parse yaml layout file {} {}", fname, e);
            return 4;
        }
    };
    let mut all_rules = Vec::new();

    for layout in layouts {
        // get desktop number
        let desktop_num = format!("{}", layout.desktop);
        // get programs
        let programs = layout.programs;
        // set up desktop
        let err_code = set_up_desktop(&desktop_num, &programs, spath, &mut all_rules);
        if err_code > 0 {
            return err_code;
        }
    }

    println!("[LOG] running rules...");
    for rule in &all_rules {
        if bspwm::send(spath, &format!("rule -r {}", &rule)) > 0 {
            return 3;
        }
    }

    return 0;
}

fn load_layout(spath: &str, args: &str) -> u8 {
    // loads a layout file and configures the system apropiately.
    let file_path = match common::get_layout_file(args) {
        Ok(path) => path,
        Err(_) => {
            println!(
                "[ERROR] can't load layout stored in \"{}\", file doesn't exsist.",
                args
            );
            return 4;
        }
    };

    let layout_file = match read_to_string(&file_path) {
        Ok(data) => data,
        Err(_) => {
            println!("[ERROR] could not layout file {}", args);
            return 4;
        }
    };

    println!("[LOG] loading layout {}", file_path);

    return if file_path.ends_with(".yml") || file_path.ends_with(".yaml") {
        load_from_yaml(layout_file, spath, &file_path)
    } else {
        load_from_layout(layout_file, spath)
    };
}

fn write_shutdown(stream: &mut UnixStream, res: u8) {
    let _ = stream.write(&[res]);
    let _ = stream.write_all(&format!("{}", res).as_bytes()).unwrap();
    // if res > 0 {
    //     stream.write_all(&format!("{}", res).as_bytes()).unwrap();
    // } else {
    //     stream.write_all(&format!("{}", res).as_bytes()).unwrap();
    // }
    let _ = stream.shutdown(std::net::Shutdown::Write);
}

fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    stream.read_to_string(&mut command).unwrap();
    let _ = stream.shutdown(std::net::Shutdown::Read);
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
        "load-layout" => load_layout(spath, args),
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

fn recv_loop(progr: &str, bspwm_socket: String) -> std::io::Result<()> {
    // println!("recv_loop");
    let listener = UnixListener::bind(progr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let tmp_bspwm = bspwm_socket.clone();
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

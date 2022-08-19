use configparser::ini::Ini;
use freedesktop_entry_parser::parse_entry;
use procfs::process;
use serde::{Deserialize, Serialize};
use serde_yaml;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::{thread, time};
use xdotool::window::get_window_pid;

mod bspwm;
mod common;
// mod free_desktop;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Program {
    name: String,
    state: Option<String>,
    delay: Option<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct DesktopLayout {
    desktop: u8,
    asyncro: Option<bool>,
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

fn get_exec(program: &Program) -> String {
    return match parse_entry(format!("/usr/share/applications/{}", &program.name)) {
        Ok(entry) => entry
            .section("Desktop Entry")
            .attr("Name")
            .expect(&program.name)
            .to_string(),
        Err(_) => program.name.clone(),
    };
}

fn run_exec(exec: &String, desktop_name: &str, program: &Program, spath: &str) -> u8 {
    let rules = [
        format!(
            "{}:{} desktop={}",
            exec[0..1].to_uppercase() + &exec[1..],
            exec,
            desktop_name
        ),
        format!(
            "{}:{} desktop={}",
            exec[0..1].to_uppercase() + &exec[1..],
            exec[0..1].to_uppercase() + &exec[1..],
            desktop_name
        ),
        format!("{}:{} desktop={}", exec, exec, desktop_name),
        format!("*:{} desktop={}", exec, desktop_name),
        format!(
            "{} desktop={}",
            exec[0..1].to_uppercase() + &exec[1..],
            desktop_name
        ),
        format!("{} desktop={}", exec, desktop_name),
    ];

    for rule in &rules {
        if bspwm::send(spath, &format!("rule -a {} follow=off -o", &rule)) > 0 {
            return 3;
        }
    }

    // thread::sleep(time::Duration::from_millis(100));

    let error_code = common::open_program(&format!("{}", &program.name));

    // bspwm::open_on_desktop(spath, &format!("{} {}", &program.name, desktop_name));
    let t = match program.delay {
        Some(times) => time::Duration::from_millis(500 * times as u64),
        None => time::Duration::from_millis(500),
    };

    if error_code > 0 {
        return error_code;
    }

    thread::sleep(t);

    for rule in &rules {
        // println!("{} | {}", exec, rule);
        if bspwm::send(spath, &format!("rule -r {}", &rule)) > 0 {
            return 3;
        }
    }

    return 0;
}

fn remove_present(progs: &Vec<Program>, execs: &mut Vec<String>) -> Vec<Program> {
    let mut programs = Vec::new();
    for program in progs {
        let prog = &get_exec(&program).to_lowercase();
        if execs.contains(&prog) {
            let i = execs.iter().position(|x| x == prog).unwrap();
            println!("removeing {} at position {}", prog, i);
            execs.remove(i);
        } else {
            programs.push(program.clone());
        }
    }

    return programs;
}

fn get_progs(desktop_name: &str, programs: &Vec<Program>, spath: &str) -> Vec<Program> {
    let res = bspwm::query(spath, &format!("query -N -d {}", desktop_name));
    let window_ids = res.trim().split('\n');
    // println!("window_ids :  {:?}", window_ids);
    let mut execs = Vec::new();
    // let prog_names = Vec::from_iter(programs.into_iter().map(|x| get_exec(x).to_lowercase()));

    for id in window_ids {
        // println!("id :  {} | d :  {}", id, desktop_name);
        let pid = match String::from_utf8(get_window_pid(id).stdout) {
            Ok(pid) => match pid.trim().parse::<i32>() {
                Ok(p) => p,
                Err(_) => 0,
            },
            Err(_) => 0,
        };

        let exec = match process::Process::new(pid) {
            Ok(proc) => match proc.cmdline() {
                Ok(path) => {
                    let p = Vec::from_iter(path[0].split('/'));
                    p[p.len() - 1].to_string()
                }
                Err(_) => String::new(),
            },
            Err(_) => String::new(),
        };
        execs.push(exec.to_lowercase());
    }

    let progs = remove_present(programs, &mut execs);

    return progs;
}

fn set_up_desktop(desktop_name: &str, programs: &Vec<Program>, spath: &str) -> Vec<u8> {
    let progs = get_progs(desktop_name, programs, spath);
    let mut ecs = Vec::new();

    for program in progs {
        let exec = get_exec(&program).to_lowercase();
        let ec = run_exec(&exec, desktop_name, &program, spath);
        ecs.push(ec);
        if ec > 0 {
            println!(
                "[ERROR] count not launch {} on desktop {}.",
                program.name, desktop_name
            );
        }
    }

    return ecs;
}

fn init_layout(spath: &str, layout: &DesktopLayout) -> Vec<u8> {
    let desktop_num = format!("{}", layout.desktop);
    let programs = layout.programs.clone();
    let tmp_spath = spath.to_string();
    set_up_desktop(&desktop_num, &programs, &tmp_spath)
}

fn init_layouts(spath: &str, layouts: &Vec<DesktopLayout>) -> Vec<u8> {
    let mut res_codes = Vec::new();

    for layout in layouts {
        res_codes.append(&mut init_layout(&spath, &layout));
    }

    res_codes
}

fn load_from_yaml(layout_file: String, spath: &str, fname: &str) -> u8 {
    let layouts: Vec<DesktopLayout> = match serde_yaml::from_str(&layout_file) {
        Ok(data) => data,
        Err(e) => {
            println!("[ERROR] could not parse yaml layout file {}: {}", fname, e);
            return 4;
        }
    };

    let mut async_layouts = Vec::new();
    let mut sync_layouts = Vec::new();

    for layout in layouts {
        match layout.asyncro {
            Some(b) if b => async_layouts.push(layout),
            _ => sync_layouts.push(layout),
        }
    }

    let mut launchers = Vec::new();

    let tmp_spath = spath.to_owned().clone();
    launchers.push(thread::spawn(move || {
        init_layouts(&tmp_spath, &sync_layouts)
    }));

    for layout in async_layouts {
        let tmp_spath = spath.to_owned().clone();
        launchers.push(thread::spawn(move || init_layout(&tmp_spath, &layout)));
    }

    for launcher in launchers {
        let err_codes = match launcher.join() {
            Ok(ecs) => ecs,
            Err(e) => {
                println!("[ERROR] got unknown error: {:?}", e);
                vec![2]
            }
        };
        for ec in err_codes {
            if ec > 0 {
                return ec;
            }
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

    // stop the window manager from following to the newest window. not actually nessesary.
    bspwm::send(&spath, "config ignore_ewmh_focus true");

    let error_code = if file_path.ends_with(".yml") || file_path.ends_with(".yaml") {
        load_from_yaml(layout_file, spath, &file_path)
    } else {
        load_from_layout(layout_file, spath)
    };

    bspwm::send(&spath, "config ignore_ewmh_focus false");

    return error_code;
}

fn write_shutdown(stream: &mut UnixStream, res: u8) {
    let _ = stream.write(&[res]);
    let _ = stream.write_all(&format!("{}", res).as_bytes()).unwrap();
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

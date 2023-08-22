use shellexpand;
use std::env::set_current_dir;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use xdotool::window::get_window_pid;
use std::{thread, time};
use procfs::process;
use std::io::{Read, Write};
use crate::common::open_program;
use crate::wm_lib;
use crate::wm_lib::{Program, DesktopLayout};
use freedesktop_entry_parser::parse_entry;
use crate::config::OptGenRes;

fn remove_present(progs: &Vec<Program>, execs: &mut Vec<String>) -> Vec<Program> {
    let mut programs = Vec::new();
    for program in progs {
        let prog = &get_exec(program).to_lowercase();
        if execs.contains(prog) {
            let i = execs.iter().position(|x| x == prog).unwrap();
            println!("removeing {} at position {}", prog, i);
            execs.remove(i);
        } else {
            programs.push(program.clone());
        }
    }

    programs
}

fn get_exec(program: &Program) -> String {
    match parse_entry(format!("/usr/share/applications/{}", &program.name)) {
        Ok(entry) => entry
            .section("Desktop Entry")
            .attr("Name")
            .expect(&program.name)
            .to_string(),
        Err(_) => program.name.clone(),
    }
}

fn get_progs(desktop_name: &str, programs: &Vec<Program>, spath: &str) -> Vec<Program> {
    let res = query(spath, &format!("query -N -d {}", desktop_name));
    let window_ids = res.trim().split('\n');
    // println!("window_ids :  {:?}", window_ids);
    let mut execs = Vec::new();
    // let prog_names = Vec::from_iter(programs.into_iter().map(|x| get_exec(x).to_lowercase()));

    for id in window_ids {
        // println!("id :  {} | d :  {}", id, desktop_name);
        let pid = match String::from_utf8(get_window_pid(id).stdout) {
            Ok(pid) => pid.trim().parse::<i32>().unwrap_or(0),
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

    remove_present(programs, &mut execs)
}

fn load_layout(spath: &str, args: &str) -> u8 {
    // loads a layout file and configures the system apropiately.
    
    let layout_yaml = match wm_lib::get_layout(args) {
        Ok(layout) => layout,
        Err(n) => return n,
    };

    // println!("[LOG] loading layout {}", file_path);

    // stop the window manager from following to the newest window. not actually nessesary.
    send(spath, "config ignore_ewmh_focus true");

    // let error_code = if file_path.ends_with(".yml") || file_path.ends_with(".yaml") {
    //     load_from_yaml(layout_file, spath, &file_path)
    // } else {
    //     load_from_layout(layout_file, spath)
    //     4
    // };

    let error_code = load_from_yaml(layout_yaml.desktops, spath);

    send(spath, "config ignore_ewmh_focus false");

    // TODO: set workspaces to desktops

    error_code
}

fn load_from_yaml(layouts: Vec<DesktopLayout>, spath: &str) -> u8 {
    let mut async_layouts = Vec::new();
    let mut sync_layouts = Vec::new();

    for layout in layouts {
        match layout.asyncro {
            Some(b) if b => async_layouts.push(layout),
            _ => sync_layouts.push(layout),
        }
    }

    let mut launchers = Vec::new();

    let tmp_spath = spath.to_string();
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

    0
}

fn init_layouts(spath: &str, layouts: &Vec<DesktopLayout>) -> Vec<u8> {
    let mut res_codes = Vec::new();

    for layout in layouts {
        res_codes.append(&mut init_layout(spath, layout));
    }

    res_codes
}

fn init_layout(spath: &str, layout: &DesktopLayout) -> Vec<u8> {
    let desktop_num = layout.desktop.to_string();
    let programs = layout.programs.clone();
    let tmp_spath = spath.to_string();
    set_up_desktop(&desktop_num, &programs, &tmp_spath)
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

    ecs
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
        if send(spath, &format!("rule -a {} follow=off -o", &rule)) > 0 {
            return 3;
        }
    }

    // thread::sleep(time::Duration::from_millis(100));

    let error_code = open_program(&program.name);

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
        if send(spath, &format!("rule -r {}", &rule)) > 0 {
            return 3;
        }
    }

    0
}

fn query(spath: &str, message: &str) -> String {
    match UnixStream::connect(spath) {
        Ok(mut stream) => {
            match stream.write_all(&make_api(message)) {
                Ok(_) => {}
                Err(e) => println!("[ERROR] couldn't not send data to BSPWM: {}", e),
            };
            let mut res = String::new();
            match stream.read_to_string(&mut res) {
                Ok(_) => {}
                Err(_e) => {
                    println!("could not read from bspwm socket. user intervention requested.");
                }
            };
            res
        }
        Err(e) => {
            println!(
                "[ERROR] could not connect to bspwm (are you usign the right socket file?): {}",
                e
            );
            String::new()
        }
    }
}

fn open_on_desktop(spath: &str, raw_args: &str) -> u8 {
    //get args
    let args = get_n_args(2, raw_args);
    if args.last() == Some(&String::new()) {
        return 7;
    }

    let (program, desktop) = (&args[0], &args[1]);

    let _ = set_current_dir(Path::new(&shellexpand::tilde("~/").to_string()));

    println!("[LOG] running {} on desktop {}:", program, desktop);

    if send(spath, &format!("desktop {} -f", desktop)) > 0 {
        return 3;
    }

    let init_nodes_n = query(spath, "query -N -d").len();

    let cmd = if program.ends_with(".desktop") {
        Command::new("gtk-launch")
            .arg(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else {
        Command::new(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };

    let process = match cmd {
        Ok(_) => {
            println!("[LOG] program {} launched", program);
            0
        }
        Err(e) => {
            println!("[ERROR] program {} could not be launched: {}", program, e);
            return 4;
        }
    };

    let t = time::Duration::from_millis(100);

    while init_nodes_n == query(spath, "query -N -d").len() {
        // println!("[DEBUG] sleeping...");
        thread::sleep(t);
    }

    process
}

fn get_n_args(n: i32, arg_str: &str) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let mut args = Vec::new();
    let a = arg_str.split_once(' ');
    match a {
        Some((arg1, arg2)) => {
            args.push(arg1.to_string());
            args.append(&mut get_n_args(n - 1, arg2));
        }
        None => args.push(arg_str.to_string()),
    };
    args
}

fn focus_on(spath: &str, destination: &str) -> u8 {
    send(spath, &format!("desktop -f {}", destination))
}

fn move_to(spath: &str, destination: &str) -> u8 {
    send(spath, &format!("node -d {}", destination))
}

fn close_focused(spath: &str) -> u8 {
    send(spath, "node -c")
}

fn make_api(message: &str) -> Vec<u8> {
    let null = &format!("{}", 0 as char);
    let mut res = message.replace(' ', null).as_bytes().to_vec();
    res.push(0);
    res
}

fn send(spath: &str, message: &str) -> u8 {
    // println!("message :  {}", message);
    match UnixStream::connect(spath) {
        Ok(mut stream) => {
            match stream.write_all(&make_api(message)) {
                Ok(_) => {}
                Err(e) => println!("[ERROR] couldn't not send data to BSPWM: {}", e),
            };
            let mut res: Vec<u8> = Vec::new();
            match stream.read_to_end(&mut res) {
                Ok(_) => {}
                Err(_e) => {
                    println!("could not read from bspwm socket. user intervention requested.");
                }
            };
            if !res.is_empty() && res[0] == 7_u8 {
                println!("[ERROR] BSPWM error: {}", String::from_utf8(res).unwrap());
                6
            } else {
                // use std::str;
                // println!("res :  {}", str::from_utf8(&res).unwrap());
                0
            }
        }
        Err(e) => {
            println!(
                "[ERROR] could not connect to bspwm (are you usign the right socket file?): {}",
                e
            );
            5
        }
    }
}

pub async fn bspwm_switch(cmd: &str, args: &str, spath: &str) -> OptGenRes {
    match cmd {
        "move-to" => Some((move_to(spath, args), None)),
        "close-focused" => Some((close_focused(spath), None)),
        "open-at" => Some((open_on_desktop(spath, args), None)),
        "focus-on" => Some((focus_on(spath, args), None)),
        "load-layout" => Some((load_layout(spath, args), None)),
        _ => None,
    }
}

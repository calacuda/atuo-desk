use std::path::PathBuf;
use std::{thread, time};
use tokio::task;
use xdg::BaseDirectories;
use tokio::fs::write;

use crate::common;
use crate::config::OptGenRes;
use crate::wm_lib;

pub async fn leftwm_switch(cmd: &str, args: &str) -> OptGenRes {
    match cmd {
        "move-to" => Some((move_to(args).await, None)),
        "close-focused" => Some((close_focused().await, None)),
        "open-at" | "open-on" => Some((open_on_desktop(args).await, None)),
        "focus-on" => Some((focus_on(args).await, None)),
        "load-layout" => Some((load_layout(args).await, None)),
        _ => None,
    }
}

pub fn get_cmd_file() -> Option<PathBuf> {
    let file_name = "command-0.pipe";
    match BaseDirectories::with_prefix("leftwm") {
        Ok(run_dir) => {
            match run_dir.find_runtime_file(&file_name) {
                Some(path) => Some(path),
                None => {
                    // println!("[ERROR] Couldn't find the leftwm command.pipe file.");
                    None
                }
            }
        }
        Err(e) => {
            println!("[ERROR] Couldn't find the leftwm run dir. got error: \n{e}");
            None
        }
    }
}

async fn load_layout(args: &str) -> u8 {
    // loads a layout file and configures the system apropiately.

    let layout_yaml = match wm_lib::get_layout(args) {
        Ok(layout) => layout,
        Err(n) => return n,
    };

    load_from_yaml(layout_yaml).await
}

async fn load_from_yaml(layouts: Vec<wm_lib::DesktopLayout>) -> u8 {
    let mut async_layouts = Vec::new();
    let mut sync_layouts = Vec::new();

    for layout in layouts {
        match layout.asyncro {
            Some(b) if b => async_layouts.push(layout),
            _ => sync_layouts.push(layout),
        }
    }

    let mut launchers = Vec::new();

    launchers.push(task::spawn(
        async move { init_layouts(&sync_layouts).await },
    ));

    for layout in async_layouts {
        launchers.push(task::spawn(async move { init_layout(&layout).await }));
    }

    for launcher in launchers {
        let err_codes = match launcher.await {
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

async fn init_layouts(layouts: &Vec<wm_lib::DesktopLayout>) -> Vec<u8> {
    let mut res_codes = Vec::new();

    for layout in layouts {
        res_codes.append(&mut init_layout(&layout).await);
    }

    res_codes
}

async fn init_layout(layout: &wm_lib::DesktopLayout) -> Vec<u8> {
    // let desktop_num = layout.desktop;
    let programs = layout.programs.clone();
    set_up_desktop(&layout.desktop, &programs).await
}

async fn set_up_desktop(desktop_name: &str, programs: &Vec<wm_lib::Program>) -> Vec<u8> {
    // let progs = get_progs(desktop_name, programs);
    let mut ecs = Vec::new();

    for (program, delay) in get_progs(programs) {
        let ec = open_on_desktop(&format!("{desktop_name} {program}")).await;
        let t = match delay {
            Some(times) => time::Duration::from_millis(500 * times as u64),
            None => time::Duration::from_millis(1000),
        };

        // this needs to be a synchronous thread interrupt to stop the program from launching something else while waiting.
        thread::sleep(t);

        ecs.push(ec);
    }

    return ecs;
}

fn get_progs(programs: &Vec<wm_lib::Program>) -> Vec<(String, Option<u8>)> {
    let mut progs = Vec::new();

    for prog in programs {
        progs.push((prog.name.clone(), prog.delay));
    }

    progs
}

async fn move_to(args: &str) -> u8 {
    // TODO: add args check (ie return 7 if to many or few)
    send_cmd(&format!("SendWindowToTag {args}")).await
}

async fn close_focused() -> u8 {
    send_cmd("CloseWindow").await
}

async fn open_on_desktop(args: &str) -> u8 {
    let (desktop, cmd) = match args.split_once(" ") {
        Some(args) => args,
        None => {
            println!("[ERROR] wrong number of arguments.");
            return 7;
        }
    };

    // TODO: read config file and get desktop i based on the treating desktop as a name.
    let desktop = match desktop.parse::<i32>() {
        Ok(i) => format!("{}", i - 1),
        Err(e) => {
            println!("[ERROR] could not interpret i as a number. got error: {e}");
            return 2;
        }
    };

    // TODO: add a way to specify workspace (ie. which monitor should go to the tag).
    let tag_switch_ec = focus_on(&desktop).await; // send_cmd(&format!("SendWorkspaceToTag 0 {desktop}")).await;
    if tag_switch_ec > 0 {
        return tag_switch_ec;
    }

    common::open_program(cmd);
    0
}

async fn focus_on(args: &str) -> u8 {
    send_cmd(&format!("SendWorkspaceToTag 0 {args}")).await
}

async fn send_cmd(cmd: &str) -> u8 {
    let file_path = match get_cmd_file() {
        Some(path) => path,
        None => {
            println!("[ERROR] Couldn't find the leftwm command.pipe file.");
            return 5;
        }
    };

    let cmd = format!("{cmd}\n");
    // let mut file = match OpenOptions::new()
    //     .append(true)
    //     .open(file_path) {
    //         Ok(file) => file,
    //         Err(e) => {
    //             println!("[ERROR] Couldn't open leftwm commands.pipe for writing: {e}");
    //             return 5;
    //         }
    //     };

    // let mut file = match File::create(file_path).await {
    //         Ok(file) => file,
    //         Err(e) => {
    //             println!("[ERROR] Couldn't open leftwm commands.pipe for writing: {e}");
    //             return 5;
    //         }
    //     };

    if let Err(e) = write(file_path, cmd.as_bytes()).await {
        println!("[ERROR] Couldn't write to leftwm commands.pipe: {e}");
        5
    } else {
        0
    }
}

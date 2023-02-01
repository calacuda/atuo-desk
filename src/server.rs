use std::fs::create_dir;
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::future::BoxFuture;
use crate::config::{GenericRes, OptGenRes};
use crate::config;
use crate::common;
use crate::qtile;
use crate::bspwm;
use crate::leftwm;
use crate::hooks;
use crate::msgs;


enum WindowManager {
    Qtile,
    BSPWM,
    LeftWM,
    NoWM,
    Headless,
}

fn make_payload(ec: u8, message: Option<String>) -> Vec<u8> {
    let mut payload = vec![ec, if ec > 0 {7} else {0}];
    match message {
        Some(mesg) => {
            // println!("making payload with message {mesg}");
            payload.append(&mut mesg.as_bytes().into_iter().map(|byte| *byte).collect::<Vec<u8>>());
        },
        None => {}
    }
    // println!("payload => {:?}", payload);
    payload
}

/// tests the function "make_payload"
#[test]
fn test_make_payload() {
    let pl_1 = make_payload(5, Some("12345".to_string()));
    let pl_2 = make_payload(0, Some("123".to_string()));
    let pl_3 = make_payload(5, None);
    let pl_4 = make_payload(0, None);
    let pl_5 = make_payload(0, Some(String::new()));
    let pl_6 = make_payload(0, Some("".to_string()));
    // assert_eq!(pl_1.len(), 7);
    assert_eq!(pl_1, vec![5, 7, 49, 50, 51, 52, 53]);
    assert_eq!(pl_2, vec![0, 0, 49, 50, 51]);
    assert_eq!(pl_3, vec![5, 7]);
    assert_eq!(pl_4, vec![0, 0]);
    assert_eq!(pl_5, vec![0, 0]);
    assert_eq!(pl_6, vec![0, 0]);
}

fn write_shutdown(stream: &mut UnixStream, ec: u8, message: Option<String>) {
    // println!("message => {:?}", message);
    let payload = make_payload(ec, message);
    // println!("payload => {:?}", payload);
    let _ = stream.try_write(&payload);
    let _ = stream.shutdown();
}

async fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    let _ = stream.read_to_string(&mut command).await;
    let _ = stream.shutdown();
    command
}

async fn switch_board<'t>(
    wm: &WindowManager,
    cmd: &'t str, 
    args: &'t str, 
    spath: &'t str, 
    maybe_hook_data: &'t mut Option<hooks::HookData>
) -> GenericRes {
    let mut futures: Vec<BoxFuture<'t, OptGenRes>> = Vec::new();
    // let mut futures: Vec<SwitchBoardFuture> = Vec::new();

    match wm { 
        WindowManager::Qtile => {
            #[cfg(feature = "qtile")]
            futures.push(Box::pin(qtile::qtile_switch(cmd, args, spath)));
        }
        WindowManager::BSPWM => {
            #[cfg(feature = "bspwm")]
            futures.push(Box::pin(bspwm::bspwm_switch(cmd, args, spath)));
        }
        WindowManager::LeftWM => {
            #[cfg(feature = "leftwm")]
            futures.push(Box::pin(leftwm::leftwm_switch(cmd, args)));
            // futures.push(Box::pin(leftwm::leftwm_switch(cmd, args, spath)));
        }
        WindowManager::Headless |
        WindowManager::NoWM => {}
    }
    // common should be checked last.
    #[cfg(feature = "common")]
    futures.push(Box::pin(common::common_switch(cmd, args)));
    #[cfg(feature = "systemctl")]
    futures.push(Box::pin(common::sysctl_switch(cmd)));
    #[cfg(feature = "media")]
    futures.push(Box::pin(common::media_switch(cmd, args)));
    #[cfg(feature = "hooks")]
    futures.push(Box::pin(hooks::hooks_switch(cmd, args, maybe_hook_data)));
    

    for future in futures {
        if let Some(res) = future.await {
            return res
        }
    }

    (1, Some(format!("there is now command by the name of, {cmd}")))
}

fn split_cmd(command: &str) -> (String, String){
    match command.split_once(' ') {
        Some((cmd, args)) => (cmd.to_owned(), args.to_owned()),
        None => (command.to_owned(), String::new()),
    }
}

// #[cfg(not(feature = "qtile"))]
async fn handle_client_gen(
    cmd: String,
    args: String,
    wm: &WindowManager,
    hooks: &mut Option<hooks::HookData>, 
    // _config_hooks: &config::Hooks, 
    mut stream: UnixStream, 
    spath: &str
) {
    // handle comand here
    let (ec, message) = switch_board(wm, &cmd, &args, spath, hooks).await;
    // let mesg = match message {
    //     Some(mesg) => mesg,
    //     None => 
    // };
    write_shutdown(&mut stream, ec, message);
    drop(stream)
}

// #[cfg(feature = "qtile")]
async fn handle_client_qtile(
    cmd: String,
    args: String,
    wm: &WindowManager,
    mut stream: UnixStream, 
    layout: &mut Option<qtile::QtileCmdData>, 
    hook_data: &mut Option<hooks::HookData>,
    wm_socket: &str,
) -> Option<qtile::QtileCmdData> {
    // handle comand here
    match qtile::qtile_api(&cmd, &args, layout).await {
        Some(qtile::QtileAPI::Layout(new_layout)) => {
            println!("[DEBUG] Response Code: 0");
            write_shutdown(&mut stream, 0, Some("configured layout".to_string()));
            drop(stream);
            Some(new_layout)
        },
        Some(qtile::QtileAPI::Message(message)) => {
            println!("[LOG] handle_qtile_client => Message");
            println!("[DEBUG] sending message => {message}");
            write_shutdown(&mut stream, 0, Some(message));
            drop(stream);
            None
        }
        Some(qtile::QtileAPI::Res(ec)) => {
            println!("[DEBUG] Response Code: {ec}");
            write_shutdown(&mut stream, ec, None);
            drop(stream);
            None
        }
        None => {
            let (ec, message) = switch_board(wm, &cmd, &args, wm_socket, hook_data).await;
            write_shutdown(&mut stream, ec, message);
            drop(stream);
            None
        }
    }
}

fn is_wm_running(procs: &System, proc_name: &str, wm: &str) -> bool {
    for proc in procs.processes_by_exact_name(proc_name) {
        // println!("{} | {:?}", proc.name(), proc.exe());
        if proc.name() == wm || proc.exe().ends_with(wm) {
            return true;
        }
    }

    return false;
}

fn get_running_wm() -> WindowManager {
    use std::env;
    // use std::path::Path;
    // println!("[ERROR] Couldn't find the leftwm command.pipe file.");
    let procs = System::new_all();

    match env::var("DISPLAY") {
        Ok(_) => {
            // let qtile_soc_fname = format!("{home}/.cache/qtile/qtilesocket.{display}");
            // println!("[DEV_DEBUG] qtile_socket_fname => {qtile_soc_fname}");

            // if Path::new(&qtile_soc_fname).exists() {
            if is_wm_running(&procs,"qtile", "qtile") {
                println!("[LOG] Running in Qtile mode");
                WindowManager::Qtile
            // } else if Path::new("/tmp/bspwm_0_0-socket").exists() {
            } else if is_wm_running(&procs, "bspwm", "bspwm") {
                println!("[LOG] Running in BSPWM mode");
                WindowManager::BSPWM
            // } else if leftwm::get_cmd_file() != None {
            } else if is_wm_running(&procs,"leftwm", "leftwm") {
                println!("[LOG] Running in leftwm mode");
                WindowManager::LeftWM
            } else {
                WindowManager::Headless
            }
        }
        _ => WindowManager::NoWM, 
    }
}

async fn recv_loop(configs: config::Config) -> std::io::Result<()> {
    // println!("recv_loop");
    let program_socket = configs.server.listen_socket.as_str();
    let wm_socket = configs.server.wm_socket.as_str();
    println!("[LOG] listening on socket: {}", program_socket);
    let listener = UnixListener::bind(program_socket)?;
    // #[cfg(feature = "qtile")]  // make this compile for all features?
    let mut layout: Option<qtile::QtileCmdData> = None;
    let (mut hooks, hook_checking) = if Some(true) == configs.hooks.listen && cfg!(feature = "hooks") {
        let (control_tx, mut control_rx) = tokio::sync::mpsc::channel::<hooks::HookDB>(1);
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<msgs::Hook>(1);
        let stop_exec = configs.hooks.exec_ignore.clone();
        let conf_hooks = configs.hooks.hooks.clone();
        // make a runtime dir for auto-desk
        let _ = create_dir(config::get_pipe_d());
        let hook_checking = task::spawn( async move {
            hooks::check_even_hooks(&mut control_rx, &mut cmd_rx, stop_exec, conf_hooks).await;
        });
        let hooks_db = hooks::HookDB::new();
        (Some(hooks::HookData { send: control_tx, cmd: cmd_tx, db: hooks_db }), Some(hook_checking))
    } else {
        (None, None)
    };
    let wm = get_running_wm();

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                /* connection succeeded */
                let command = read_command(&mut stream).await;
                // println!("command: {}", command);
                let (cmd, args) = split_cmd(&command);
                if cmd == "SERVER-EXIT" {
                    break;
                }

                match wm {
                    WindowManager::Qtile => {
                        // #[cfg(feature = "qtile")]
                        // TODO: implement "SERVER-STOP" check.
                        match handle_client_qtile(cmd, args, &wm, stream, &mut layout, &mut hooks, wm_socket).await {
                            Some(lo) => {
                                layout = Some(lo.clone());
                                println!("[DEBUG] layout: {:?}", lo);
                                task::spawn(
                                    async move {
                                        for program in lo.queue {
                                            common::open_program(&program);
                                        }
                                    }
                                );
                            }
                            None => {}
                        }
                    }
                    WindowManager::BSPWM | 
                    WindowManager::LeftWM |
                    WindowManager::Headless |
                    WindowManager::NoWM => {
                        // #[cfg(not(feature = "qtile"))]
                        {
                            // let tmp_wms = wm_socket.to_string();
                            // let tmp_hooks = hooks.clone();
                            // let tmp_config_hooks = configs.hooks.clone();
                            handle_client_gen(cmd, args, &wm, &mut hooks, stream, wm_socket).await;
                        }
                    }
                }
            }
            Err(err) => {
                println!("[ERROR] could not except socket connection. {:#?}", err);
                /* connection failed */
                break;
            }
        }
    }

    println!("[LOG] killing listener");
    drop(listener);
    println!("[LOG] listener killed");
    println!("[LOG] stopping hooks");
    if let Some(h_dat) = hooks {
        let _ = h_dat.cmd.send(msgs::Hook::Exit).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    if let Some(hook_checking) = hook_checking {
        hook_checking.abort()
    }
    println!("[LOG] hooks stopped");
    Ok(())
}

pub async fn server_start() {
    let configs = match config::get_configs() {
        Ok(configs) => configs,
        Err(e) => {
            println!("[ERROR] could not load configs. reason: {e}");
            println!("now exiting");
            return;
        }
    };
    let prog_so = &configs.server.listen_socket;
    // let wm_socket = &configs.server.wm_socket;

    // println!("{:#?}", configs);
    // println!("progr {}\nwm_socket {}", prog_so, wm_socket);
    let p = std::path::Path::new(&prog_so);
    if p.exists() {
        // println!("program socket exists");
        std::fs::remove_file(prog_so).unwrap_or_else(|e| 
            {
                println!("[ERROR] could not delete previous socket at {:?}\ngot error:\n{}", &prog_so, e);
                panic!(""); 
            }
        )
    };

    match recv_loop(configs).await {
        Ok(_) => {}
        Err(e) => println!("[ERROR] {}", e),
    }
    println!("[LOG] server session terminated");
}

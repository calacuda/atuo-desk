use crate::bspwm;
use crate::common;
use crate::config;
use crate::config::{GenericRes, OptGenRes};
use crate::hooks;
use crate::leftwm;
use crate::msgs;
use crate::qtile;
use futures::future::BoxFuture;
use std::fs::create_dir;
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task;

#[derive(PartialEq)]
pub enum WindowManager {
    Qtile,
    Bspwm,
    LeftWM,
    NoWM,
    Headless,
}

fn make_payload(ec: u8, message: Option<String>) -> Vec<u8> {
    let mut payload = vec![ec, if ec > 0 { 7 } else { 0 }];
    if let Some(mesg) = message {
        // println!("making payload with message {mesg}");
        payload.append(&mut mesg.as_bytes().to_vec());
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

async fn write_shutdown(stream: &mut UnixStream, ec: u8, message: Option<String>) {
    // println!("message => {:?}", message);
    let payload = make_payload(ec, message);
    // println!("payload => {:?}", payload);
    if let Err(reason) = stream.try_write(&payload) {
        println!("[ERROR] could not write out to client because: \"{reason}\", attempting to close communication stream");
    }
    if let Err(reason) = stream.shutdown().await {
        println!(
            "[ERROR] could not shutdown after write because: \"{reason}\", client will likely hang"
        );
    };
}

async fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    let _ = stream.read_to_string(&mut command).await;
    command
}

async fn switch_board<'t>(
    wm: &WindowManager,
    cmd: &'t str,
    args: &'t str,
    spath: &'t str,
    maybe_hook_data: &'t mut Option<hooks::HookData>,
    layout: &'t mut qtile::QtileCmdData,
) -> GenericRes {
    let mut futures: Vec<BoxFuture<'t, OptGenRes>> = Vec::new();
    // let mut futures: Vec<SwitchBoardFuture> = Vec::new();

    match wm {
        WindowManager::Qtile => {
            #[cfg(feature = "qtile")]
            futures.push(Box::pin(qtile::qtile_switch(cmd, args, spath, layout)));
        }
        WindowManager::Bspwm => {
            #[cfg(feature = "bspwm")]
            futures.push(Box::pin(bspwm::bspwm_switch(cmd, args, spath)));
        }
        WindowManager::LeftWM => {
            #[cfg(feature = "leftwm")]
            futures.push(Box::pin(leftwm::leftwm_switch(cmd, args)));
            // futures.push(Box::pin(leftwm::leftwm_switch(cmd, args, spath)));
        }
        WindowManager::Headless | WindowManager::NoWM => {}
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
            return res;
        }
    }

    (
        1,
        Some(format!("there is now command by the name of, {cmd}")),
    )
}

fn split_cmd(command: &str) -> (String, String) {
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
    spath: &str,
    layout: &mut qtile::QtileCmdData,
) {
    // handle comand here
    let (ec, message) = switch_board(wm, &cmd, &args, spath, hooks, layout).await;
    // let mesg = match message {
    //     Some(mesg) => mesg,
    //     None =>
    // };
    write_shutdown(&mut stream, ec, message).await;
    drop(stream)
}

// #[cfg(feature = "qtile")]
async fn handle_client_qtile(
    cmd: String,
    args: String,
    wm: &WindowManager,
    mut stream: UnixStream,
    layout: &mut qtile::QtileCmdData,
    hook_data: &mut Option<hooks::HookData>,
    spath: &str,
) -> Option<qtile::QtileCmdData> {
    // handle comand here
    match qtile::qtile_api(&cmd, &args, layout).await {
        Some(qtile::QtileAPI::Layout(new_layout)) => {
            println!("[DEBUG] Response Code: 0");
            write_shutdown(&mut stream, 0, Some("configured layout".to_string())).await;
            drop(stream);
            Some(new_layout)
        }
        Some(qtile::QtileAPI::Message(message)) => {
            println!("[LOG] handle_qtile_client => Message");
            println!("[DEBUG] sending message => {message}");
            write_shutdown(&mut stream, 0, Some(message)).await;
            drop(stream);
            None
        }
        Some(qtile::QtileAPI::Res(ec)) => {
            println!("[DEBUG] Response Code: {ec}");
            write_shutdown(&mut stream, ec, None).await;
            drop(stream);
            None
        }
        None => {
            let (ec, message) = switch_board(wm, &cmd, &args, spath, hook_data, layout).await;
            write_shutdown(&mut stream, ec, message).await;
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

    false
}

pub fn get_running_wm() -> WindowManager {
    use std::env;
    // use std::path::Path;
    // println!("[ERROR] Couldn't find the leftwm command.pipe file.");
    let procs = System::new_all();

    match env::var("DISPLAY") {
        Ok(_) => {
            // let qtile_soc_fname = format!("{home}/.cache/qtile/qtilesocket.{display}");
            // println!("[DEV_DEBUG] qtile_socket_fname => {qtile_soc_fname}");

            // if Path::new(&qtile_soc_fname).exists() {
            if is_wm_running(&procs, "qtile", "qtile") {
                println!("[LOG] Running in Qtile mode");
                WindowManager::Qtile
            // } else if Path::new("/tmp/bspwm_0_0-socket").exists() {
            } else if is_wm_running(&procs, "bspwm", "bspwm") {
                println!("[LOG] Running in BSPWM mode");
                WindowManager::Bspwm
            // } else if leftwm::get_cmd_file() != None {
            } else if is_wm_running(&procs, "leftwm", "leftwm") {
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
    let mut layout: qtile::QtileCmdData = qtile::QtileCmdData::new();

    let (mut hooks, hook_checking) =
        if Some(true) == configs.hooks.listen && cfg!(feature = "hooks") {
            let (control_tx, mut control_rx) = tokio::sync::mpsc::channel::<hooks::HookDB>(1);
            let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<msgs::EventCmd>(1);
            let stop_exec = configs.hooks.exec_ignore.clone();
            let conf_hooks = configs.hooks.hooks.clone();
            // make a runtime dir for auto-desk
            let _ = create_dir(config::get_pipe_d());
            let hook_checking = task::spawn(async move {
                hooks::check_even_hooks(
                    &mut control_rx,
                    &mut cmd_rx,
                    stop_exec,
                    conf_hooks,
                    configs.hooks.ignore_web,
                )
                .await;
            });
            let hooks_db = hooks::HookDB::new();
            (
                Some(hooks::HookData {
                    send: control_tx,
                    cmd: cmd_tx,
                    db: hooks_db,
                }),
                Some(hook_checking),
            )
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
                        if let Some(lo) = handle_client_qtile(
                            cmd,
                            args,
                            &wm,
                            stream,
                            &mut layout,
                            &mut hooks,
                            program_socket,
                        )
                        .await
                        {
                            layout = lo.clone();
                            println!("[DEBUG] layout: {:?}", lo);
                            task::spawn(async move {
                                for program in lo.queue {
                                    common::open_program(&program);
                                }
                            });
                        }
                    }
                    WindowManager::Bspwm
                    | WindowManager::LeftWM
                    | WindowManager::Headless
                    | WindowManager::NoWM => {
                        // #[cfg(not(feature = "qtile"))]
                        {
                            // let tmp_wms = wm_socket.to_string();
                            // let tmp_hooks = hooks.clone();
                            // let tmp_config_hooks = configs.hooks.clone();
                            handle_client_gen(
                                cmd,
                                args,
                                &wm,
                                &mut hooks,
                                stream,
                                wm_socket,
                                &mut layout,
                            )
                            .await;
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

    println!("[LOG] killing unix socket");
    drop(listener);
    println!("[LOG] unix socket killed");
    println!("[LOG] stopping event listeners");
    if let Some(h_dat) = hooks {
        let _ = h_dat.cmd.send(msgs::EventCmd::Exit).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    if let Some(hook_checking) = hook_checking {
        hook_checking.abort()
    }
    println!("[LOG] event listeners stopped");
    Ok(())
}

fn clear_sockets(prog_so: &str) {
    for p in [prog_so, &config::get_pipe_f()] {
        let path = std::path::Path::new(&p);
        if path.exists() {
            // println!("program socket exists");
            println!("[LOG] clearing socker file at {:?}", path);
            if let Err(e) = std::fs::remove_file(path)
            // .unwrap_or_else(|e|
            {
                println!(
                    "[ERROR] could not delete previous socket at {:?}\ngot error:\n{}",
                    &path, e
                );
                // panic!("");
            }
            // )
        }
    }
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
    let prog_so = configs.server.listen_socket.clone();
    // let wm_socket = &configs.server.wm_socket;

    // println!("{:#?}", configs);
    // println!("progr {}\nwm_socket {}", prog_so, wm_socket);
    clear_sockets(&prog_so);

    match recv_loop(configs).await {
        Ok(_) => {}
        Err(e) => println!("[ERROR] {}", e),
    }

    clear_sockets(&prog_so);
    println!("[LOG] server session terminated");
}

#![warn(clippy::all)]
#![feature(type_alias_impl_trait)]
use tokio::net::{UnixListener, UnixStream};
use tokio::task;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::macros::support::Future;
// use std::future::Future;
use futures::future::BoxFuture;
#[cfg(feature = "hooks")]
use tokio::sync::mpsc::channel;

// type MainFuncs<'a> = Vec<&'a dyn Fn(&str, &str, &str) -> Option<u8>>;
// type SwitchBoardFuture = impl Future<Output = Option<GenericRes>>;
// type SwitchBoardFutures = Vec<SwitchBoardFuture>;

// TODO: make this just Layout(qtile::QtileCmdData), Res(u8), 
//       and replace, Location(String) and Clear(bool) wth a "Message" variant.
#[cfg(feature = "qtile")]
enum QtileAPI {
    Layout(qtile::QtileCmdData),
    Message(String),
    Res(u8),
}

// #[cfg(feature = "hooks")]
type GenericRes = (u8, Option<String>);
type OptGenRes = Option<GenericRes>;

// enum GenericRes {
//     Message((u8, String)),
//     Res(u8),
// }

fn make_payload(ec: u8, message: Option<String>) -> Vec<u8> {
    let mut payload = vec![ec, if ec > 0 {7} else {0}];
    match message {
        Some(mesg) => {let _ = mesg.as_bytes().into_iter().map(|byte| payload.push(*byte));},
        None => {}
    }
    payload
}

#[cfg(feature = "test")]
/// tests the function "make_payload"
fn test_make_payload() {
    let pl_1 = make_payload(5, Some("12345"));
    let pl_2 = make_payload(0, Some("123"));
    let pl_3 = make_payload(5, None);
    let pl_4 = make_payload(0, None);
    let pl_5 = make_payload(0, String::new());
    let pl_6 = make_payload(0, "");
    // assert_eq!(pl_1.len(), 7);
    assert_eq!(pl_1, vec![5, 7, 49, 50, 51, 52, 53]);
    assert_eq!(pl_2, vec![0, 0, 49, 50, 51]);
    assert_eq!(pl_3, vec![5, 7]);
    assert_eq!(pl_4, vec![0, 0]);
    assert_eq!(pl_5, vec![0, 0]);
    assert_eq!(pl_6, vec![0, 0]);
}

fn write_shutdown(stream: &mut UnixStream, ec: u8, message: Option<String>) {
    let payload = make_payload(ec, message);
    let _ = stream.try_write(&payload);
    let _ = stream.shutdown();
}

// #[cfg(feature = "qtile")]
// fn writes_shutdown(stream: &mut UnixStream, res_code: u8, mesg: &str) {
//     let _ = stream.try_write(&[res_code]);
//     let _ = stream.try_write(mesg.as_bytes());
//     // let _ = stream.write_all(mesg.as_bytes());
//     let _ = stream.shutdown();
// }

async fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    let _ = stream.read_to_string(&mut command).await;
    let _ = stream.shutdown();
    command
}

#[cfg(feature = "common")]
async fn common_switch(cmd: &str, args: &str, _spath: &str) -> OptGenRes{
    match cmd {
        "open-here" => Some((common::open_program(args), None)),
        "screen-shot" => Some((common::screen_shot(), None)),
        "inc-bl" => Some((common::backlight::inc_bright(args), None)),
        "dec-bl" => Some((common::backlight::dec_bright(args), None)),
        "add-monitor" => Some((common::xrandr::add_monitor(args), None)),
        _ => None,
    }
}

#[cfg(feature = "media")]
async fn media_switch(cmd: &str, args: &str, _spath: &str) -> OptGenRes {
    match cmd {
        "vol-up" => Some((common::media::volume_up(args), None)),
        "vol-down" => Some((common::media::volume_down(args), None)),
        "mute" => Some((common::media::mute(), None)),
        "play/pause" => Some((common::media::play_pause(), None)),
        "play-track" => Some((common::media::play(), None)),
        "pause-track" => Some((common::media::pause(), None)),
        "stop-track" => Some((common::media::stop(), None)),
        "next-track" => Some((common::media::next_track(), None)),
        "last-track" => Some((common::media::last_track(), None)),
        _ => None,
    }
}

#[cfg(feature = "systemctl")]
async fn sysctl_switch(cmd: &str, _args: &str, _spath: &str) -> OptGenRes {
    match cmd {
        "poweroff" => Some((common::power::power_off(), None)),
        "hibernate" => Some((common::power::hibernate(), None)),
        "reboot" => Some((common::power::reboot(), None)),
        "sleep" | "suspend" => Some((common::power::sleep(), None)),
        "lock" => Some((common::power::lock(), None)),
        "logout" => Some((common::power::logout(), None)),
        _ => None,
    }
}

#[cfg(feature = "bspwm")]
async fn bspwm_switch(cmd: &str, args: &str, spath: &str) -> OptGenRes {
    match cmd {
        "move-to" => Some((bspwm::move_to(spath, args), None)),
        "close-focused" => Some((bspwm::close_focused(spath), None)),
        "open-at" => Some((bspwm::open_on_desktop(spath, args), None)),
        "focus-on" => Some((bspwm::focus_on(spath, args), None)),
        "load-layout" => Some((bspwm::load_layout(spath, args), None)),
        _ => None,
    }
}

#[cfg(feature = "qtile")]
async fn qtile_switch(cmd: &str, args: &str, spath: &str) -> OptGenRes {
    match cmd {
        // "move-to" => Some(qtile::move_to(spath, args)),
        // "close-focused" => Some(qtile::close_focused(spath)),
        "open-at" | "open-on" => Some((qtile::open_on_desktop(spath, args), None)),
        "focus-on" => Some((qtile::focus_on(spath, args), None)),
        _ => None,
    }
}

#[cfg(feature = "qtile")]
async fn qtile_api(
    cmd: &str, 
    args: &str, 
    layout: &mut Option<qtile::QtileCmdData>
) -> Option<QtileAPI> {
    match cmd {
        // "load-layout" => Some(QtileAPI::layout(qtile::load_layout(args))),
        "load-layout" => {
            match qtile::make_cmd_data(args) {
                Ok(layout) => Some(QtileAPI::Layout(layout)),
                Err(ec) => Some(QtileAPI::Res(ec)),
            }
        }
        "auto-move" => Some(
            match qtile::auto_move(args, layout) {
                Ok(Some(loc)) => QtileAPI::Message(loc),
                Ok(None) => QtileAPI::Res(0), 
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        "should-clear" => Some(
            match qtile::should_clear(args, layout) {
                Ok(to_clear_or_not_to_clear) => QtileAPI::Message(if to_clear_or_not_to_clear {"true"} else {"false"}.to_string()), // that is the question
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        _ => None,
    }
}

async fn hooks_switch( 
    cmd: &str, 
    args: &str, 
    maybe_hook_data: &mut Option<hooks::HookData>,
) -> OptGenRes {
    match (cmd, maybe_hook_data, ) { 
        ( "add-hook", Some(hook_data) )=> Some((hooks::add_hook(args, hook_data).await, None)),
        ( "rm-hook", Some(hook_data) )=> Some((hooks::rm_hook(args, &mut hook_data.db).await, None)),
        ( "ls-hook" | "list-hook", Some(hook_data) )=> {
            // TODO: this chould be a table, just like sql output. Thats why its called table.
            let table = hooks::get_hook(&hook_data.db).await;
            Some((0, Some(table)))
        },
        _ => None,
    }
}

// async fn switch_on_hooks() -> {

// }

async fn switch_board<'t>(
    cmd: &'t str, 
    args: &'t str, 
    spath: &'t str, 
    maybe_hook_data: &'t mut Option<hooks::HookData>
) -> GenericRes {
    let mut futures: Vec<BoxFuture<'t, OptGenRes>> = Vec::new();
    // let mut futures: Vec<SwitchBoardFuture> = Vec::new();

    #[cfg(feature = "qtile")]
    futures.push(Box::pin(qtile_switch(cmd, args, spath)));
    #[cfg(feature = "bspwm")]
    futures.push(Box::pin(bspwm_switch(cmd, args, spath)));

    // common should be checked last.
    #[cfg(feature = "common")]
    futures.push(Box::pin(common_switch(cmd, args, spath)));
    #[cfg(feature = "systemctl")]
    futures.push(Box::pin(sysctl_switch(cmd, args, spath)));
    #[cfg(feature = "media")]
    futures.push(Box::pin(media_switch(cmd, args, spath)));
    #[cfg(feature = "hooks")]
    futures.push(Box::pin(hooks_switch(cmd, args, maybe_hook_data)));
    

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

#[cfg(not(feature = "qtile"))]
async fn handle_client_gen(
    hooks: &mut Option<hooks::HookData>, 
    // _config_hooks: &config::Hooks, 
    mut stream: UnixStream, 
    spath: &str
) {
    // TODO: implement hooks for this and switch board
    let command = read_command(&mut stream).await;
    // println!("{}", command);
    let (cmd, args) = split_cmd(&command);


    // handle comand here
    let (ec, message) = switch_board(&cmd, &args, spath, hooks).await;
    // let mesg = match message {
    //     Some(mesg) => mesg,
    //     None => 
    // };
    write_shutdown(&mut stream, ec, message);
    drop(stream)
}

#[cfg(feature = "qtile")]
async fn handle_client_qtile(
    mut stream: UnixStream, 
    layout: &mut Option<qtile::QtileCmdData>, 
    hook_data: &mut Option<hooks::HookData>,
    wm_socket: &str,
) -> Option<qtile::QtileCmdData> {
    let command = read_command(&mut stream).await;
    println!("command: {}", command);
    let (cmd, args) = split_cmd(&command);

    // handle comand here
    match qtile_api(&cmd, &args, layout).await {
        Some(QtileAPI::Layout(new_layout)) => {
            println!("[DEBUG] Response Code: 0");
            write_shutdown(&mut stream, 0, Some("configured layout".to_string()));
            drop(stream);
            Some(new_layout)
        },
        Some(QtileAPI::Message(message)) => {
            println!("[DEBUG] sending message => {message}");
            write_shutdown(&mut stream, 0, Some(message));
            drop(stream);
            None
        }
        Some(QtileAPI::Res(ec)) => {
            println!("[DEBUG] Response Code: {ec}");
            write_shutdown(&mut stream, ec, None);
            drop(stream);
            None
        }
        None => {
            let (ec, message) = switch_board(&cmd, &args, wm_socket, hook_data).await;
            write_shutdown(&mut stream, ec, message);
            drop(stream);
            None
        }
    }
}

async fn recv_loop(configs: config::Config) -> std::io::Result<()> {
    // println!("recv_loop");
    let program_socket = configs.server.listen_socket.as_str();
    let wm_socket = configs.server.wm_socket.as_str();
    println!("[LOG] listening on socket: {}", program_socket);
    let listener = UnixListener::bind(program_socket)?;
    #[cfg(feature = "qtile")]
    let mut layout: Option<qtile::QtileCmdData> = None;
    // let (control_tx, events_rx, hooks_db) = if cfg!(feature = "hooks") {
    let mut hooks: Option<hooks::HookData> = if cfg!(feature = "hooks") {
        // let (events_tx, mut events_rx) = channel::<hooks::HookDB>(1);
        let (control_tx, mut control_rx) = channel::<hooks::HookDB>(1);
        // hooks::start_event_checkers(command_rx, events_tx);
        let stop_exec = configs.hooks.exec_ignore.clone();
        let conf_hooks = configs.hooks.hooks.clone();
        task::spawn( async move {
            hooks::check_even_hooks(&mut control_rx, stop_exec, conf_hooks).await;
        });
        let hooks_db = hooks::HookDB::new();
        // TODO: load config file hooks here
        Some(hooks::HookData { send: control_tx, db: hooks_db })
        // Some((control_tx, events_rx, hooks_db))
    } else {
        None
    };

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                /* connection succeeded */
                #[cfg(feature = "qtile")]
                match handle_client_qtile(stream, &mut layout, &mut hooks, wm_socket).await {
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
                #[cfg(not(feature = "qtile"))]
                {
                    // let tmp_wms = wm_socket.to_string();
                    // let tmp_hooks = hooks.clone();
                    // let tmp_config_hooks = configs.hooks.clone();
                    handle_client_gen(&mut hooks, stream, wm_socket).await;
                }
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

#[tokio::main]
async fn main() -> Result<(), ()> {
    let configs = match config::get_configs() {
        Ok(configs) => configs,
        Err(e) => {
            println!("{e}");
            return Err(());
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
    // println!("Goodbye!");
    Ok(())
}

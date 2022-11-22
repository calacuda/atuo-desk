use configparser::ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
// use std::os::unix::net::{UnixListener, UnixStream};
use tokio::net::{UnixListener, UnixStream};
// #[cfg(not(feature = "qtile"))]
use tokio::task;
// use std::thread;
// use std::io::{Read, Write};
// use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "common")]
use common;
#[cfg(feature = "bspwm")]
use bspwm;
#[cfg(feature = "qtile")]
use qtile;

// TODO: make this just Layout(qtile::QtileCmdData), Res(u8), 
//       and replace, Location(String) and Clear(bool) wth a "Message" variant.
#[cfg(feature = "qtile")]
enum QtileAPI {
    Layout(qtile::QtileCmdData),
    Location(String),
    Clear(bool),
    Res(u8),
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
        None => panic!("config file has no server configurations. exiting"),
    };
}

fn write_shutdown(stream: &mut UnixStream, res: u8) {
    let _ = stream.try_write(&[res]);
    // let _ = stream.write_all(&format!("{}", res).as_bytes()).unwrap();
    let _ = stream.shutdown();
}

#[cfg(feature = "qtile")]
fn writes_shutdown(stream: &mut UnixStream, res_code: u8, mesg: &str) {
    let _ = stream.try_write(&[res_code]);
    let _ = stream.try_write(mesg.as_bytes());
    // let _ = stream.write_all(mesg.as_bytes());
    let _ = stream.shutdown();
}

async fn read_command(stream: &mut UnixStream) -> String {
    let mut command = String::new();
    // stream.set_nonblocking(false);
    let _ = stream.read_to_string(&mut command).await;
    let _ = stream.shutdown();
    return command;
}

#[cfg(feature = "common")]
fn common_switch(cmd: &str, args: &str, _spath: &str) -> Option<u8> {
    match cmd {
        "open-here" => Some(common::open_program(args)),
        "screen-shot" => Some(common::screen_shot()),
        "inc-bl" => Some(common::backlight::inc_bright(args)),
        "dec-bl" => Some(common::backlight::dec_bright(args)),
        "add-monitor" => Some(common::xrandr::add_monitor(args)),
        _ => None,
    }
}

#[cfg(feature = "media")]
fn media_switch(cmd: &str, args: &str, _spath: &str) -> Option<u8> {
    match cmd {
        "vol-up" => Some(common::media::volume_up(args)),
        "vol-down" => Some(common::media::volume_down(args)),
        "mute" => Some(common::media::mute()),
        "play/pause" => Some(common::media::play_pause()),
        "play-track" => Some(common::media::play()),
        "pause-track" => Some(common::media::pause()),
        "stop-track" => Some(common::media::stop()),
        "next-track" => Some(common::media::next_track()),
        "last-track" => Some(common::media::last_track()),
        _ => None,
    }
}

#[cfg(feature = "systemctl")]
fn sysctl_switch(cmd: &str, _args: &str, _spath: &str) -> Option<u8> {
    match cmd {
        "poweroff" => Some(common::power::power_off()),
        "hibernate" => Some(common::power::hibernate()),
        "reboot" => Some(common::power::reboot()),
        "sleep" | "suspend" => Some(common::power::sleep()),
        "lock" => Some(common::power::lock()),
        "logout" => Some(common::power::logout()),
        _ => None,
    }
}

#[cfg(feature = "bspwm")]
fn bspwm_switch(cmd: &str, args: &str, spath: &str) -> Option<u8> {
    match cmd {
        "move-to" => Some(bspwm::move_to(spath, args)),
        "close-focused" => Some(bspwm::close_focused(spath)),
        "open-at" => Some(bspwm::open_on_desktop(spath, args)),
        "focus-on" => Some(bspwm::focus_on(spath, args)),
        "load-layout" => Some(bspwm::load_layout(spath, args)),
        _ => None,
    }
}

#[cfg(feature = "qtile")]
fn qtile_switch(cmd: &str, args: &str, spath: &str) -> Option<u8> {
    match cmd {
        // "move-to" => Some(qtile::move_to(spath, args)),
        // "close-focused" => Some(qtile::close_focused(spath)),
        "open-at" | "open-on" => Some(qtile::open_on_desktop(spath, args)),
        "focus-on" => Some(qtile::focus_on(spath, args)),
        _ => None,
    }
}

#[cfg(feature = "qtile")]
fn qtile_api(cmd: &str, args: &str, layout: &mut Option<qtile::QtileCmdData>) -> Option<QtileAPI> {
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
                Ok(Some(loc)) => QtileAPI::Location(loc),
                Ok(None) => QtileAPI::Res(0), 
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        "should-clear" => Some(
            match qtile::should_clear(args, layout) {
                Ok(to_clear_or_not_to_clear) => QtileAPI::Clear(to_clear_or_not_to_clear), // that is the question
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        _ => None,
    }
}

async fn switch_board(cmd: &str, args: &str, spath: &str) -> u8 {

    let mut fs: Vec<&dyn Fn(&str, &str, &str) -> Option<u8>> = Vec::new();

    #[cfg(feature = "qtile")]
    fs.push(&qtile_switch);
    #[cfg(feature = "bspwm")]
    fs.push(&bspwm_switch);

    // common should be checked last.
    #[cfg(feature = "common")]
    fs.push(&common_switch);
    #[cfg(feature = "systemctl")]
    fs.push(&sysctl_switch);
    #[cfg(feature = "media")]
    fs.push(&media_switch);

    for f in fs {
        match f(&cmd, &args, spath) {
            Some(res) => return res,
            None => {}
        }
    }

    1
}

fn split_cmd(command: &str) -> (String, String){
    match command.split_once(" ") {
        Some((cmd, args)) => (cmd.to_owned(), args.to_owned()),
        None => (command.to_owned(), String::new()),
    }
}

#[cfg(not(feature = "qtile"))]
async fn handle_client_gen(mut stream: UnixStream, spath: String) {
    let command = read_command(&mut stream).await;
    // println!("{}", command);
    let (cmd, args) = split_cmd(&command);


    // handle comand here
    let res: u8 = switch_board(&cmd, &args, &spath).await;
    write_shutdown(&mut stream, res);
    drop(stream)
}

#[cfg(feature = "qtile")]
async fn handle_client_qtile(mut stream: UnixStream, layout: &mut Option<qtile::QtileCmdData>, wm_socket: &str) -> Option<qtile::QtileCmdData> {
    let command = read_command(&mut stream).await;
    println!("command: {}", command);
    let (cmd, args) = split_cmd(&command);

    // handle comand here
    let api_res = qtile_api(&cmd, &args, layout);
    match api_res {
        Some(QtileAPI::Layout(layout)) => {
            println!("[DEBUG] Response Code: 0");
            write_shutdown(&mut stream, 0);
            drop(stream);
            Some(layout)
        },
        Some(QtileAPI::Location(location)) => {
            println!("[DEBUG] location: {location}");
            writes_shutdown(&mut stream, 0, &location);
            drop(stream);
            None
        }
        Some(QtileAPI::Res(ec)) => {
            println!("[DEBUG] Response Code: {ec}");
            write_shutdown(&mut stream, ec);
            drop(stream);
            None
        }
        Some(QtileAPI::Clear(should_clear)) => {
            println!("[DEBUG] should clear workspace '{args}'? {}.", if should_clear {"yes"} else {"no"});
            writes_shutdown(&mut stream, 0, if should_clear {"true"} else {"false"});
            drop(stream);
            None
        }
        None => {
            let res: u8 = switch_board(&cmd, &args, wm_socket).await;
            write_shutdown(&mut stream, res);
            drop(stream);
            None
        }
    }
}

async fn recv_loop(program_socket: &str, wm_socket: &str) -> std::io::Result<()> {
    // println!("recv_loop");
    println!("[LOG] listening on socket: {}", program_socket);
    let listener = UnixListener::bind(program_socket)?;
    #[cfg(feature = "qtile")]
    let mut layout: Option<qtile::QtileCmdData> = None;

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                /* connection succeeded */
                #[cfg(feature = "qtile")]
                match handle_client_qtile(stream, &mut layout, wm_socket).await {
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
                    let tmp_wms = wm_socket.to_string();
                    task::spawn(
                        async move {
                            handle_client_gen(stream, tmp_wms)
                        }
                    );
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
async fn main() -> Result<(), Box<dyn Error>> {
    let configs: HashMap<String, Option<String>> =
        get_servers(&"~/.config/desktop-automater/config.ini");
    let (prog_so, wm_socket) = match (configs.get("listen-socket"), configs.get("wm-socket")) {
        (Some(Some(p)), Some(Some(b))) => (p.to_owned(), b.to_owned()),
        (Some(Some(p)), None) => {
            println!("bspwm socket location not specified, using default.");
            (p.to_owned(), "/tmp/QTILE_SOC".to_string())
        }
        (None, Some(Some(b))) => {
            println!("program socket location not specified, using default.");
            ("/tmp/desktop-automater".to_string(), b.to_owned())
        }
        (None, None) => {
            println!("neither socket location were specified, using defaults.");
            (
                "/tmp/desktop-automater".to_string(),
                "/tmp/QTILE_SOC".to_string(),
            )
        }
        _ => panic!("configs corrupted. the configuration options needs to be set or not present. manual editing is adviced")
    };
    // println!("{:#?}", configs);
    // println!("progr {}\nwm_socket {}", prog_so, wm_socket);
    let p = std::path::Path::new(&prog_so);
    if p.exists() {
        // println!("program socket exists");
        std::fs::remove_file(&prog_so).expect(&format!(
            "could not delete previous socket at {:?}",
            &prog_so
        ));
    }
    match recv_loop(&prog_so, &wm_socket).await {
        Ok(_) => {}
        Err(e) => println!("[ERROR] {}", e),
    }
    // println!("Goodbye!");
    Ok(())
}

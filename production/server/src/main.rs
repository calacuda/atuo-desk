use configparser::ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use unix_socket::Incoming;

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

fn handle_client(mut stream: UnixStream) {
    // let (client, soc_adr) = match stream.accept() {
    //     Ok((c, s)) => c, s,
    //     Err(err) => panic!("got error : {:#?}", err),
    // };
    // stream.accept();
    // stream.connect();
    let mut response = String::new();
    stream.set_nonblocking(false);
    stream.read_to_string(&mut response).unwrap();
    stream.shutdown(std::net::Shutdown::Read);
    println!("{}", response);
    stream.write_all(b"hello world").unwrap();
    stream.shutdown(std::net::Shutdown::Write);
    drop(stream)
}

fn recv_loop(progr: &str, bspwm: &str) -> std::io::Result<()> {
    println!("recv_loop");
    let listener = UnixListener::bind(progr)?;

    // loop {
    //     println!("loop");
    //     match listener.accept() {
    //         Ok((mut socket, addr)) => {
    //             println!("{:#?}", socket);
    //             let mut response = String::new();
    //             socket.read_to_string(&mut response)?;
    //             // socket.write_all(b"hello world")?;
    //             println!("{}", response);
    //         }
    //         Err(e) => {
    //             println!("accept function failed: {:?}", e);
    //             break;
    //         }
    //     }
    // }

    for stream in listener.incoming() {
        // println!("for");
        // listener.accept();
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                thread::spawn(|| handle_client(stream));
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

fn main() {
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
        std::fs::remove_file(&prog_so);
        // .with_context(|| format!("could not delete previous socket at {:?}", &progr_so))?;
    }
    recv_loop(&prog_so, &bspwm_so);
}

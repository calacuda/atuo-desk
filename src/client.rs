// #![deny(clippy::all)]
use crate::config;
use crate::server::{get_running_wm, WindowManager};
use clap::ArgMatches;
use log::{error, info};
use std::fs::remove_file;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::exit;
use std::str;

type ErrorCode = u8;

/// entry point to client.rs  when running a subcommand other then "stop"
pub fn handle_args(args: ArgMatches) {
    // let args = get_args();
    let subargs = args.subcommand().unwrap();
    let configs = match config::get_configs() {
        Ok(configs) => configs,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    let server_soc: String = configs.server.listen_socket;

    match subargs.0 {
        "launch" => handle_launch(subargs.1.to_owned(), server_soc),
        "layout" => handle_layout(subargs.1.to_owned(), server_soc),
        _ => panic!("argument not yet implemented!"),
    }
}

/// entry point to client.rs when running subcommand "stop"
/// stops the running server and cleans up the file system
pub async fn stop_server() {
    let configs = match config::get_configs() {
        Ok(configs) => configs,
        Err(e) => {
            error!("loading configs produced error: {}", e);
            error!("failed to stop server to to previous errors");
            return;
        }
    };

    let server_soc: String = configs.server.listen_socket;

    let _ = kill_server(&server_soc);
    match clean_fs(&server_soc) {
        Ok(_) => {}
        Err(failed_files) => {
            error!("failed to delete the following files: {:?}", failed_files);
        }
    }
}

/// send a kill signal to the server.
fn kill_server(server_soc: &str) -> Result<String, String> {
    // send kill signal to unix socket server located at server_soc.
    // TODO: implement the SERVER-EXIT command.
    let (ec, res_text) = send_data(String::from("SERVER-EXIT"), server_soc);
    if ec > 0 {
        Err(res_text)
    } else {
        Ok(res_text)
    }
}

/// removes unused files from the file system (eg. the server socket, and the ports sentinel com file. )
fn clean_fs(server_soc: &str) -> Result<(), Vec<String>> {
    let f_s = [server_soc.to_string(), config::get_pipe_f()];
    let mut e_s = Vec::new();

    // remove the runtime dir which stores named pipes and such.
    // let _ = remove_dir_all(config::get_pipe_d());

    // remove all files outside the runtime dir. if it can't rm the file for what ever
    // reason, it will add the files it can't remove to a vector.
    for f in f_s {
        if let Err(e) = remove_file(&f) {
            e_s.push(format!("failed to clear file \"{f}\". got error:{e}"));
        };
    }

    if e_s.is_empty() {
        Ok(())
    } else {
        Err(e_s)
    }
}

fn send_data(data: String, server_soc: &str) -> (ErrorCode, String) {
    let mut stream = match UnixStream::connect(server_soc) {
        Ok(stream) => stream,
        Err(_) => {
            error!("couldn't connect to socket at \"{}\"", server_soc);
            info!("Error Hints:");
            info!(" - Do you have the socket configured corectly?");
            info!(" - Is the server running?");
            exit(1);
        }
    };

    match stream.write_all(&data.into_bytes()) {
        Ok(_) => {}
        Err(e) => {
            error!("sending data to server produced error: {e}");
            exit(1);
        }
    };

    match stream.shutdown(Shutdown::Write) {
        Ok(_) => {}
        Err(e) => {
            error!("shutting down write access to socket produced error: \"{e}\"");
            error!("program will now hang.");
        }
    };

    let mut response_bytes = Vec::new();
    match stream.read_to_end(&mut response_bytes) {
        Ok(_) => {}
        Err(e) => {
            error!("reading server response resulted in error: \"{e}\"");
            exit(1);
        }
    };

    let (ec, response) = if !response_bytes.is_empty() {
        let ec = response_bytes[0];
        let response = &response_bytes[1..];
        (ec, response)
    } else {
        error!("server gave no response check server logs.");
        return (7, String::new());
    };

    if ec > 0 {
        // print!("{}", 7 as char);
        error!(
            "The server reported an error (check 'systemctl status' for message). error code: {ec}"
        );
    }

    let res_text = String::from_utf8_lossy(response);
    info!("server responded \"{res_text}\"");
    (ec, String::from(res_text))
}

fn handle_layout(args: ArgMatches, server_soc: String) {
    let input_layout_fname: String = args.get_one::<String>("layout").unwrap().clone();
    let layout_path = input_layout_fname; // find_layout(input_layout_fname);
    info!(
        "loading the {} layout...",
        Path::new(&layout_path).to_str().unwrap()
    );

    let (_ec, _response_bytes) = send_data(format!("load-layout {}", layout_path), &server_soc);
}

fn handle_launch(args: ArgMatches, server_soc: String) {
    let program = args.get_one::<String>("program").unwrap().clone();
    info!("launching {}...", program);

    let payload = if get_running_wm() == WindowManager::Qtile && args.contains_id("desktop") {
        let Some(wm_class) = args.get_one::<String>("wm-class") else {
            error!("the \"--wm-class\"/\"-c\" arguemnt is required when running in Qtile mode");
            return
        };

        format!(
            "open-at {} {} {}",
            program,
            wm_class,
            args.get_one::<String>("desktop").unwrap()
        )
    } else if args.contains_id("desktop") {
        format!(
            "open-at {} {}",
            program,
            args.get_one::<String>("desktop").unwrap()
        )
    } else {
        format!("open-here {}", program)
    };

    send_data(payload, &server_soc);
}

#![deny(clippy::all)]
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::exit;
use std::str;

fn main() {
    let args = get_args();
    let subargs = args.subcommand().unwrap();
    let configs = match config::get_configs() {
        Ok(configs) => configs,
        Err(e) => {
            println!("{}", e);
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

fn send_data(data: String, server_soc: String) -> Vec<u8> {
    let mut stream = match UnixStream::connect(&server_soc) {
        Ok(stream) => stream,
        Err(_) => {
            println!("[ERROR] couldn't connect to socket at \"{}\"", server_soc);
            println!("Hints:");
            println!(" - Do you have the socket configured corectly?");
            println!(" - Is the server runing?");
            exit(1);
        }
    };

    match stream.write_all(&data.into_bytes()) {
        Ok(_) => {}
        Err(e) => {
            println!("could not send data to server.");
            println!("[DEBUG] :  {}", e);
            exit(1);
        }
    };

    match stream.shutdown(Shutdown::Write) {
        Ok(_) => {}
        Err(e) => {
            println!("failed to shutdown write access to socket file.");
            println!("program will now hang.");
            println!("[DEBUG] :  {}", e);
        }
    };

    let mut response_bytes = Vec::new();
    match stream.read_to_end(&mut response_bytes) {
        Ok(_) => {}
        Err(e) => {
            println!("could not read response from server.");
            println!("[DEBUG] :  {}", e);
            exit(1);
        }
    };

    let ec = response_bytes[0];
    let response = &response_bytes[1..];

    if ec > 0 {
        print!("{}", 7 as char);
        println!("[ERROR] The server reported an error (check 'systemctl status' for message). error code: {ec}");
    } else {
        println!("[SUCCESS] responce: ")
    }

    match str::from_utf8(response) {
        Ok(text) => println!("{}", text),
        Err(_e) => {
            println!("responce had invalid UTF-8, could not parse.");
            println!("raw bytes:");
            println!("{:?}", response);
        }
    };

    response_bytes
}

fn handle_layout(args: ArgMatches, server_soc: String) {
    let input_layout_fname: String = args.get_one::<String>("layout").unwrap().clone();
    let layout_path = input_layout_fname; // find_layout(input_layout_fname);
    println!(
        "loading the {} layout...",
        Path::new(&layout_path).to_str().unwrap()
    );

    let _response_bytes = send_data(format!("load-layout {}", layout_path), server_soc);
}

fn handle_launch(args: ArgMatches, server_soc: String) {
    let program = args.get_one::<String>("program").unwrap().clone();
    println!("launching {}...", program);

    let _response_bytes = if args.contains_id("desktop") {
        send_data(
            format!(
                "open-at {} {}",
                program,
                args.get_one::<String>("desktop").unwrap()
            ),
            server_soc,
        )
    } else {
        send_data(format!("open-here {}", program), server_soc)
    };
}

fn get_args() -> ArgMatches {
    App::new("auto-desk")
        .version("0.5.0")
        .author("Calacuda. <https://github.com/calacuda>")
        .about("used to control a linux desktop running BSPWM.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("layout")
                .help("configure the system with a layout.yaml file")
                .arg(
                    Arg::new("layout")
                        // .short('l')
                        // .long("layout")
                        .value_name("LAYOUT.yml")
                        .help("the yaml file describing the desiered desktop configuration.")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("launch")
                .help("launch a program")
                .arg(
                    Arg::new("desktop")
                        .short('d')
                        .long("desktop")
                        .value_name("TARGET-DESKTOP")
                        .help("The desktop to launch the program on")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::new("program")
                        .value_name("PROGRAM")
                        .help("The program to be launched")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches()
}

// fn find_layout(fname: String) -> String {
//     /*
//      * finds the desired layout file either in the layout dir, cwd, or the path provided.
//      */
//     let paths = vec![
//         format!("~/.config/desktop-automater/layouts/{}.yml", fname),
//         format!("~/.config/desktop-automater/layouts/{}.yaml", fname),
//         fname.clone(),
//         format!("~/.config/desktop-automater/layouts/{}", fname),
//     ];
//
//     for fp in paths
//         .into_iter()
//         .map(|x| shellexpand::full(&x).unwrap().to_string())
//     {
//         if Path::new(&fp).exists() {
//             return std::fs::canonicalize(&fp.clone())
//                 .unwrap()
//                 .into_os_string()
//                 .into_string()
//                 .unwrap();
//         }
//     }
//     println!("could not find the layout file named {}", fname);
//     exit(1);
// }

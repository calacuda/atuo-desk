use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use configparser::ini::Ini;
use shellexpand;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::exit;
use std::str;

const CONFIG_ADR: &str = "~/.config/desktop-automater/config.ini";

fn main() {
    let args = get_args();
    let subargs = args.subcommand().unwrap();
    let server_soc: String = get_server_soc();

    match subargs.0 {
        "launch" => handle_launch(subargs.1.to_owned(), server_soc),
        "loadout" => handle_loadout(subargs.1.to_owned(), server_soc),
        _ => panic!("clap missed that argument! please submit a bug report."),
    }
}

// fn find_loadout(fname: String) -> String {
//     /*
//      * finds the desired loadout file either in the loadout dir, cwd, or the path provided.
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
//     println!("could not find the loadout file named {}", fname);
//     exit(1);
// }

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

    let mut response = Vec::new();
    match stream.read_to_end(&mut response) {
        Ok(_) => {}
        Err(e) => {
            println!("could not read response from server.");
            println!("[DEBUG] :  {}", e);
            exit(1);
        }
    }; //  &mut response);

    return response; // return response;
}

fn handle_loadout(args: ArgMatches, server_soc: String) {
    let input_loadout_fname: String = args.get_one::<String>("loadout").unwrap().clone();
    let loadout_path = input_loadout_fname; // find_loadout(input_loadout_fname);
    println!(
        "loading the {} loadout...",
        Path::new(&loadout_path).to_str().unwrap()
    );

    let response_bytes = send_data(format!("load-layout {}", loadout_path), server_soc);
    let ec = response_bytes[0];
    let response = &response_bytes[1..];

    if ec > 0 {
        print!("{}", 7 as char);
        print!("[ERROR] A server side error ocured. error code: ");
    } else {
        print!("[SUCCESS] responce: ")
    }
    // println!("{:#?}", response_bytes);
    match str::from_utf8(&response) {
        Ok(text) => println!("{}", text),
        Err(_e) => {
            println!("responce had invalid UTF-8, could not parse.");
            println!("raw bytes:");
            println!("{:?}", response);
        }
    };
}

fn handle_launch(_args: ArgMatches, _server_soc: String) {
    println!("launching program...");
}

fn get_servers(config_file: &str) -> HashMap<String, Option<String>> {
    let configs: HashMap<String, HashMap<String, Option<String>>> = match load_config(config_file) {
        Ok(data) => data,
        Err(e) => {
            println!("got error : {:?}", e);
            exit(1);
        }
    };

    // println!("{:#?}", configs);

    return match configs.get("server") {
        Some(data) => data.to_owned(),
        None => {
            println!("config file has no server configurations. exiting");
            exit(1);
        }
    };
}

fn get_server_soc() -> String {
    return match get_servers(CONFIG_ADR).get("prog-so") {
        Some(servers) => match servers.to_owned() {
            Some(socket_adr) => socket_adr,
            None => {
                println!("programming socket adress is absent from the config file,");
                println!("using the default \"/tmp/desktop-automater\"");
                println!("to fix this add the following line to the config:");
                println!("[SERVER]");
                println!("prog-so = /tmp/desktop-automater");
                "/tmp/desktop-automater".to_owned()
            }
        },
        None => {
            println!("Can't locate default config file using the default programming");
            println!("socket location \"/tmp/desktop-automater\"");
            println!(
                "to fix this make a config file at \"{}\" and add the following",
                CONFIG_ADR
            );
            println!("lines to the file:");
            println!("[SERVER]");
            println!("bspwm-so = /tmp/bspwm_0_0-socket");
            println!("prog-so = /tmp/desktop-automater");
            "/tmp/desktop-automater".to_owned()
        }
    };
}

fn load_config(
    config_file: &str,
) -> Result<HashMap<String, HashMap<String, Option<String>>>, Box<dyn Error>> {
    let mut config = Ini::new();
    let map = config.load(shellexpand::tilde(config_file).to_string())?;
    return Ok(map);
}

fn get_args() -> ArgMatches {
    return App::new("auto-desk")
        .version("0.1.0")
        .author("Calacuda. <https://github.com/calacuda>")
        .about("used to control a linux desktop running BSPWM.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("loadout")
                .help("configure the system with a loadout.yaml file")
                .arg(
                    Arg::new("loadout")
                        // .short('l')
                        // .long("loadout")
                        .value_name("LOADOUT.yml")
                        .help("the yaml file describing the desiered desktop configuration.")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("launch")
                .help("launch a program")
                .arg(
                    Arg::new("program")
                        // .short('p')// .short('p')
                        // .long("program")
                        // .long("program")
                        .value_name("PROGRAM")
                        .help("The program to be launched")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("desktop")
                        .short('d')
                        .long("desktop")
                        .value_name("TARGET-DESKTOP")
                        .help("The desktop to launch the program on")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .get_matches();
}

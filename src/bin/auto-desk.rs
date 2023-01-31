#![warn(clippy::all)]
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use tokio;

use auto_desk::server;
use auto_desk::client;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let args = get_args();
    
    match &args.subcommand() {
        Some(("start", _)) => server::server_start().await,
        Some((_, _)) => client::handle_args(args),
        None => {
            println!("no command specified.");
            std::process::exit(1);
        }
    }
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
        .subcommand(
            SubCommand::with_name("start")
                .help("starts the server")
        )
        .get_matches()
}
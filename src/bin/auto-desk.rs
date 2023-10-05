#![warn(clippy::all)]
use auto_desk::client;
use auto_desk::server;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use fern::colors::{Color, ColoredLevelConfig};
use log::error;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let args = get_args();
    logger_init();

    match &args.subcommand() {
        Some(("start", _)) => server::server_start().await,
        Some(("stop", _)) => client::stop_server().await,
        Some((_, _)) => client::handle_args(args),
        None => {
            error!("no command specified.");
            std::process::exit(1);
        }
    }

    std::process::exit(0);
}

fn get_args() -> ArgMatches {
    App::new("auto-desk")
        .version("0.5.0")
        .author("Calacuda. <https://github.com/calacuda>")
        .about("used to control a linux desktop running BSPWM.")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("layout")
                .about("configure the system with a layout.yaml file")
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
                .about("launch a single program")
                .help("launch a single program")
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
                    Arg::new("wm-class")
                        .short('c')
                        .long("wm-class")
                        .value_name("WM-CLASS")
                        .help("The wm-class of the window")
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
                .about("starts the server")
                .help("starts the server"),
        )
        .subcommand(
            SubCommand::with_name("stop")
                .about("stops the server and cleans up the file system.")
                .help("stops the server and cleans up the file system."),
        )
        .get_matches()
}

fn logger_init() {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Magenta)
        .error(Color::Red);

    #[cfg(debug_assertions)]
    let res = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("ptdb"))
        .chain(std::io::stderr())
        // .chain(fern::log_file("output.log")?)
        .apply();

    #[cfg(not(debug_assertions))]
    let res = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .filter(|metadata| metadata.target().starts_with("ptdb"))
        .chain(std::io::stderr())
        // .chain(fern::log_file("output.log")?)
        .apply();

    if let Err(reason) = res {
        eprintln!("failed to initiate logger because {reason}");
    } else {
        log::debug!("logger initiated");
    }
}

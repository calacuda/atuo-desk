use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::{thread, time};

fn query(spath: &str, message: &str) -> String {
    match UnixStream::connect(spath) {
        Ok(mut stream) => {
            match stream.write_all(&make_API(message)) {
                Ok(_) => {}
                Err(e) => println!("[ERROR] couldn't not send data to BSPWM: {}", e),
            };
            let mut res = String::new();
            stream.read_to_string(&mut res);
            return res;
        }
        Err(e) => {
            println!(
                "[ERROR] could not connect to bspwm (are you usign the right socket file?): {}",
                e
            );
            String::new()
        }
    }
}

pub fn open_on_desktop(spath: &str, raw_args: &str) -> u8 {
    //get args
    let args = get_n_args(2, raw_args);
    if args.last() == Some(&String::new()) {
        return 7;
    }

    println!("[LOG] running {} on desktop {}:", args[0], args[1]);

    if send(spath, &format!("desktop {} -f", args[1])) > 0 {
        return 3;
    }

    let mut process = Command::new(&args[0])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    return 0;
}

fn get_n_args(n: i32, arg_str: &str) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let mut args = Vec::new();
    let mut tmp_arg_str = arg_str.to_string();
    let a = arg_str.split_once(" ");
    match a {
        Some((arg1, arg2)) => {
            args.push(arg1.to_string());
            args.append(&mut get_n_args(n - 1, arg2));
        }
        None => args.push(arg_str.to_string()),
    };
    return args;
}

fn get_one_arg(args: &str) -> (&str, &str) {
    return match args.split_once(" ") {
        Some(cmd_args) => cmd_args.to_owned(),
        None => (args, ""), //panic!("This function take more then one input!"),
    };
}

pub fn focus_on(spath: &str, destination: &str) -> u8 {
    return send(spath, &format!("desktop -f {}", destination));
}

pub fn move_to(spath: &str, destination: &str) -> u8 {
    return send(spath, &format!("node -d {}", destination));
}

pub fn close_focused(spath: &str) -> u8 {
    return send(spath, "node -c");
}

fn make_API(message: &str) -> Vec<u8> {
    let null = &format!("{}", 0 as char);
    let mut res = message.replace(' ', null).as_bytes().to_vec();
    res.push(0);
    return res;
}

fn send(spath: &str, message: &str) -> u8 {
    match UnixStream::connect(spath) {
        Ok(mut stream) => {
            match stream.write_all(&make_API(message)) {
                Ok(_) => {}
                Err(e) => println!("[ERROR] couldn't not send data to BSPWM: {}", e),
            };
            let mut res: Vec<u8> = Vec::new();
            stream.read_to_end(&mut res);
            return if res.len() > 0 && res[0] == 7 as u8 {
                println!("[ERROR] BSPWM error: {}", String::from_utf8(res).unwrap());
                6
            } else {
                0
            };
        }
        Err(e) => {
            println!(
                "[ERROR] could not connect to bspwm (are you usign the right socket file?): {}",
                e
            );
            return 5;
        }
    }
}

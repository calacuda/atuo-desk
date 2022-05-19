use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::{thread, time};

pub fn open_on_desktop(spath: &str, raw_args: &str) -> u8 {
    print!("[LOG] running progarm on deskotp: ");

    //get args
    let args = get_n_args(2, raw_args);
    if args.last() == Some(&String::new()) {
        return 7;
    }
    println!("{} {}", args[0], args[1]);

    //make tmp desktop
    //move to tmp desktop
    let prelaunch = (
        send(spath, "monitor -a Desktop"),
        send(spath, "desktop Desktop -f"),
    );
    println!("pre {:?}", prelaunch);

    if prelaunch.0 + prelaunch.1 > 0 {
        return 3;
    }
    //open program
    // println!("[LOG] running: {}", args);
    let mut process = Command::new(&args[0]).spawn(); //.expect("failed to execute process");
    thread::sleep(time::Duration::from_millis(250));
    println!("[LOG] ran {:?}", process);
    //move to spath, destination
    //kill tmp_desktop
    let postlaunch = (
        send(spath, &format!("node -d {} --follow", args[1])),
        send(spath, "desktop Desktop --remove"),
    );
    println!("post {:?}", postlaunch);
    if postlaunch.0 + postlaunch.1 > 0 {
        return 3;
    }

    return 0;
}

fn get_n_args(n: i32, arg_str: &str) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    let mut args = Vec::new();
    let mut tmp_arg_str = arg_str.to_string();

    // for _ in 0..n {
    //     let (arg1, arg2) = get_one_arg(&tmp_arg_str);
    //     tmp_arg_str = arg2.to_string().to_owned();
    //     args.push(arg1.to_string());
    // }
    let a = arg_str.split_once(" "); // get_one_arg(arg_str)
                                     // args.push(arg1.to_string());
    match a {
        Some((arg1, arg2)) => {
            args.push(arg1.to_string());
            args.append(&mut get_n_args(n - 1, arg2));
        }
        None => args.push(arg_str.to_string()),
    };

    // if args.last() == Some(&String::new()) {
    //     println!("[ERROR] too few args given:")
    // }

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

pub fn send(spath: &str, message: &str) -> u8 {
    match UnixStream::connect(spath) {
        Ok(mut stream) => {
            match stream.write_all(&make_API(message)) {
                Ok(_) => {}
                Err(e) => println!("[ERROR] count not send data to BSPWM: {}", e),
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

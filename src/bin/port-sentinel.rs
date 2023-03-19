use auto_desk::{
    MSG_ERROR as ERROR,
    MSG_SUCCESS as SUCCESS,
    MSG_DELIM as DELIM
};
use core::time::Duration;
use std::io::Write;
use auto_desk::config::get_pipe_f;
use std::fs;
use std::fs::File;
use std::process::exit;
use std::thread::sleep;
// use std::io::Read;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use regex::Regex;

const TRACE_DIR: &str = "/sys/kernel/tracing";
const ON_FILE: &str = "tracing_on";
const CUR_TRACER: &str = "current_tracer";
const EVENT: &str = "inet_sock_set_state"; 
const SET: &str = "set_event"; 
const TRACE_F: &str = "trace"; 
const TRACER: &str = "nop"; 
// const : &str = ""; 
const NET_IN: &str = "INCOMING"; 
const NET_OUT: &str = "OUT-GOING"; 
const NET_LOC: &str = "LOCAL"; 
 
/// clears the trace buffer located at "/sys/kernel/tracing/tracer".
fn clear_trace() -> Result<(), String> {
    let trace_f = format!("{TRACE_DIR}/{TRACE_F}");

    match File::create(&trace_f) {
        Ok(f) => {
            if let Err(e) = f.set_len(0) {
                return Err(format!("could not clear the trace file at, '{trace_f}'. got error:\n{e}"))
            }
        }
        Err(e) => return Err(format!("could not write to the trace file at, '{trace_f}'. got error:\n{e}")),
    };

    Ok(())
}

/// reads the trace buffer from "/sys/kernel/tracing/tracer".
fn read_trace() -> Result<Vec<String>, String> {
    let trace_f = format!("{TRACE_DIR}/{TRACE_F}");

    let f = match fs::read_to_string(&trace_f) {
        Ok(contents) => contents,
        Err(e) => return Err(format!("could not read from file, '{trace_f}'. got error:\n{e}")),
    };
    clear_trace()?;

    Ok(f.split('\n').map(|x| x.to_string()).collect())
}

/// returns the local port of the connection. gives error if it can't find one
/// this is necessary bc the ftrace output identifies source and destination ports.
/// but we need to know which port is local.
fn make_msg(line: &str, pid: &str) -> Result<Option<String>, String> {
    let re = match Regex::new(r"sport=([0-9]+) dport=([0-9]+)") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("[ERROR] internal regex error {e}");
            return Err(format!("internal regex error {e}"));
        }
    };

    // println!("{line}");
    // println!("{:?}", re.captures(line));
    
    let (source_p, dest_p) = match re.captures(line) {
        Some(cap) if cap.len() == 3 => (cap[1].to_string(), cap[2].to_string()),
        _ => return Err("no ports could be found".to_string()),
    };

    // println!("s_port: {source_p}, d_port: {dest_p}");

    let re = match Regex::new(r" saddr=([^\s]+) daddr=([^\s]+)") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("[ERROR] internal regex error {e}");
            return Err(format!("internal regex error {e}"));
        }
    };

    // println!("{line}");
    // println!("{:?}", re.captures(line));

    let (source_ip, dest_ip) = match re.captures(line) {
        Some(cap) if cap.len() == 3 => (cap[1].to_string(), cap[2].to_string()),
        _ => return Err("source ip could be found".to_string()),
    };

    let mut ips: Vec<String> = vec!["0.0.0.0".to_string()];

    ips.append(&mut match local_ip_address::linux::list_afinet_netifas() {
        Ok(inet) => inet.into_iter().map(|x| format!("{}", x.1) ).collect(),
        Err(e) => return Err(format!("could not get IP addresses. not looking for ports. got error:\n{e}")),
    });

    ips.push("0:0:0:0:0:0:0:1".to_string());
    // println!("s_ip :  {source_ip}");
    // println!("line :  {line}");
    // println!("ips  :  {:?}", ips);

    let (l_ip, l_port, r_ip, r_port, direction) = if ips.contains(&source_ip) && ips.contains(&dest_ip) {
        (source_ip, source_p, dest_ip, dest_p, NET_LOC)
    } else if ips.contains(&source_ip) {
        (source_ip, source_p, dest_ip, dest_p, NET_OUT)
    } else {
        (dest_ip, dest_p, source_ip, source_p, NET_IN)
    };

    // println!("{l_port}");

    match l_port.parse::<u16>() {
        Ok(port_n) if port_n >= 7 => {
            // println!("port => {port_n}");
            Ok(Some([pid, &l_ip, &l_port, &r_ip, &r_port, direction].join(&format!("{DELIM}"))))
        }
        Err(e) => Err(format!("could not interpret string port number as a u16. got error: {e}")),
        _ => Ok(None),
    }
}

/// parses a single line from the tracer output, in the format of:
/// {pid}{DELIM}{local ip}{DELIM}{local port}{DELIM}{remote_ip}{DELIM}{remote port}{DELIM}{INCOMING/OUT-GOING}
/// TODO: add tcp state to line.
fn parse_line(line: &str) -> Result<Option<String>, String> {
    // return Ok(Some(format!("{exec}{DELIM}{pid}{DELIM}{port}")));
    let re = match Regex::new(r"(([^\s]+)*[^\s]+)-(([0-9]+)*[0-9]+)[ ]+\[") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("[ERROR] internal regex error {e}");
            return Err(format!("internal regex error {e}"));
        }
    };

    // let lport = get_lport(line)?;

    match re.captures(line) {
        Some(cap) => {
            // println!("{} {}", &cap[1], &cap[3]);
            make_msg(line, &cap[3])
        },
        None => Ok(None),
    }
} 

/// handles getting new traces. returns a vector of messages in the 
/// format of either, "{error-code}{DELIM}{exec}{DELIM}{pid}{DELIM}{port}",
/// or, "{error-code}{DELIM}{error_message}".
fn trace_loop() -> Vec<String> {
    let mut messages = Vec::new();

    // read TRACE_F
    let lines = match read_trace() {
        Ok(lines) => lines,
        Err(mesg) => {
            eprintln!("{mesg}");
            messages.push(mesg);
            return messages;
        }
    };

    for line in lines {
        // pull exec, PID, and port
        if !line.starts_with('#') && !line.is_empty() { 
            let mesg = match parse_line(&line) {
                Ok(Some(data)) => format!("{SUCCESS}{DELIM}{data}"),
                Ok(None) => continue,
                Err(msg) => {
                    eprintln!("[ERROR] {msg}");
                    format!("{ERROR}{DELIM}{msg}")
                },
            };
            // println!("{:#?}", mesg);
            messages.push(mesg);
        }
    }

    messages
}

/// util function used to overwrite the contests of `file` with `mesg`. `file_desc` is used in error messages. 
fn write_file(file: &str, mesg: &str, file_desc: &str) -> Result<(), String> {
    match File::create(file) {
        Ok(mut f) => {
            match write!(f, "{mesg}") {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("could not write to {file_desc} file at, '{file}'. got error:\n{e}"))
            }
        },
        Err(e) => Err(format!("could not open {file_desc} file at, '{file}'. got error:\n{e}"))
    }
}

/// sends data to auto-desk via a named pipe at, get_pipe_f(), (usually, "/run/user/<UID>/auto-desk.pipe")
fn send_data(mesg: &str) -> Result<(), String> {
    let pipe_f = get_pipe_f();

    let mut stream = match UnixStream::connect(&pipe_f) {
        Ok(stream) => stream,
        Err(_) => return Ok(()),
    };

    match stream.write_all(mesg.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[ERROR] could not send data to server. got error :\n{}", e);
            return Err(format!("[ERROR] could not send data to server. got error :\n{}", e));
        }
    };

    if let Err(e) = stream.shutdown(Shutdown::Write) {
        eprintln!("[ERROR] could not close port sentinel socket located at, {pipe_f}.");
        eprintln!("\tgot socket error {e}");
    };

    Ok(())
}

/// turns off tracing of kernel functions.
fn nop_tracer() -> Result<(), String> {
    write_file(&make_path(CUR_TRACER), TRACER, "tracer selector")
}

/// sets the event that ftrace listens for. (in this case the event will be socket sys-calls)
fn set_event() -> Result<(), String> {
    write_file(&make_path(SET), &format!("{EVENT}\n"), "event selector")
}

/// ensure that ftrace is on.
fn enable_tracing() -> Result<(), String> {
    write_file(&make_path(ON_FILE), "1", "tracer enable")
}

fn make_path(f_name: &str) -> String {
    format!("{TRACE_DIR}/{f_name}")
}

/// prepares ftrace to trace what we want and only what we want.
fn prepare_tracer() -> Result<(), String> {
    set_event()?;
    nop_tracer()?;
    enable_tracing()?;
    clear_trace()?;

    Ok(())
}

fn main() {
    if let Err(mesg) = prepare_tracer() {
        eprintln!("[ERROR] {mesg}");
        eprintln!("[FATAL ERROR] could not prepare tracer. not running auto-desk port-sentinel.");
        exit(1);
    }

    loop {
        for mesg in trace_loop() {
            // eprintln!("[LOG] sending message: {i} => {:#?}", mesg);
            if let Err(err_message) = send_data(&mesg) {
                eprintln!("[ERROR] {err_message}");
            }
        }
        
        sleep(Duration::from_millis(200));
    } 
}
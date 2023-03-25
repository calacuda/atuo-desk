use crate::{
    MSG_ERROR as ERROR,
    // MSG_SUCCESS as SUCCESS,
    MSG_DELIM as DELIM
};
use procfs::process::Process;
// use tokio::process::Command;
use tokio::time::{sleep, Duration};
// use tokio::io::{BufReader, AsyncBufReadExt};
// use tokio::fs::File;
// use tokio::io::AsyncReadExt;
// use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::mpsc::UnboundedSender;
use usb_enumeration::UsbDevice;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::PathBuf;
// use std::io::Read;
use tokio::net::UnixListener;
// use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use online::tokio::check;
use tokio::fs;
use tokio::time;
// use usb_enumeration;
use btleplug::api::{Central, CentralEvent, Manager as _,};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use iw::interfaces;
use crate::config;

// use notify::{Event, RecommendedWatcher, PollWatcher, RecursiveMode, Watcher, Config};
// use futures::{std::os::unix::fs::PermissionsExt;
//     channel::mpsc::{channel, Receiver},
//     SinkExt, StreamExt,
// };

pub type Context = HashMap<String, String>;
const RESOLUTION: u64 = 2500;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Port {
    local_addr: String, 
    remote_addr: String,
    lport: String,
    rport: String,
    state: String, 
    // inode: u64, 
    pid: Option<i32>, 
    exec: Option<String>,
    // port: String,
    con_dir: String,
}

// #[cfg(feature = "rw_test")]
// pub async fn file_exists() -> Context {
//     use std::path::Path;

//     let fname = "/tmp/flag";
//     let mut context = HashMap::new();
//     context.insert("file".to_string(), fname.to_string());
//     let exists = Path::new(fname).exists();
//     loop {
//         sleep(Duration::from_millis(RESOLUTION)).await;
//         if exists != Path::new(fname).exists() {
//             return context;
//         }
//     }
// }

pub async fn network_connection(return_tx: UnboundedSender<Context>) {
    // println!("net connected");

    let mut connected = check(None).await.is_ok();
    
    loop {
        sleep(Duration::from_millis(RESOLUTION * 4)).await;
        let tmp_connected = check(None).await.is_ok();
        if tmp_connected != connected {
            let mut context = HashMap::new();
            context.insert(
                "became".to_string(), 
                if tmp_connected {"connected".to_string()} else {"disconnected".to_string()}
            );
            match return_tx.send(context) {
                Ok(_) => {},
                Err(e) => println!("{e}\n[ERROR] could not send network connection information."),
            };
            connected = tmp_connected;
        }

    }
}

fn get_name() -> String {
    match interfaces() {
        Ok(interfaces) => {
            for interface in interfaces {
                if let Ok(ssid) =  interface.get_connected_essid() {
                    return String::from_utf8_lossy(ssid.as_bytes()).to_string()
                }
            }
            String::new()
        },
        Err(e) => {
            println!("[ERROR] error in event, no network interfaces found. {e}");
            String::new()
        }
    }
}

pub async fn wifi_change(return_tx: UnboundedSender<Context>) {
    // println!("wifi change");
    let mut old_ssid = get_name();
    loop {
        sleep(Duration::from_millis(RESOLUTION * 3)).await;
        let new_ssid = get_name();
        if new_ssid != old_ssid {
            let mut context = HashMap::new();
            context.insert("old_network".to_string(), old_ssid);
            context.insert("new_network".to_string(), new_ssid.clone());
            match return_tx.send(context) {
                Ok(_) => {},
                Err(e) => println!("{e}\n[ERROR] could not send updated wifi information context.")
            } 
            old_ssid = new_ssid;
        }
    }
}

async fn get_bckl_perc(backlight_dir: &fs::DirEntry) -> Result<f64, std::io::Error> {
    let mut brightnes_f = std::path::PathBuf::from("/sys/class/backlight/");
    brightnes_f.push(backlight_dir.file_name());
    brightnes_f.push("brightness");
    let mut max_f = std::path::PathBuf::from("/sys/class/backlight/");
    max_f.push(backlight_dir.file_name());
    max_f.push("max_brightness");

    // println!("brightness_fname => {:?}", brightnes_f.as_os_str());
    // println!("max_brightness_fname => {:?}", max_f.as_os_str());

    let current_brightness_f = match String::from_utf8(fs::read(brightnes_f).await?) {
        Ok(b) => b.replace('\n', ""),
        Err(e) => panic!("{}", e),
    };
    let max_brightness_f = match String::from_utf8(fs::read(max_f).await?){
        Ok(b) => b.replace('\n', ""),
        Err(e) => panic!("{}", e),
    };
    // println!("current_brightness_f => {:?}", current_brightness_f);
    // println!("max_brightness_f => {:?}", max_brightness_f);
    let current_brightness = current_brightness_f.parse::<i64>().unwrap();
    let max_brightness = max_brightness_f.parse::<i64>().unwrap();

    Ok(current_brightness as f64/ max_brightness as f64)
} 

pub async fn backlight_change(return_tx: UnboundedSender<Context>) {
    // println!("backlight change");

    let backlight = match tokio::fs::read_dir("/sys/class/backlight/").await {
        Ok(mut dirs) => {
            match dirs.next_entry().await {
                Ok(Some(dir)) => dir,
                _ => {
                    println!("[ERROR] could not find '/sys/class/backlight/'.");
                    return;
                } 
            }
        }
        Err(_) => {
            println!("[ERROR] back light event could not read \"/sys/class/backlight\" directory.");
            return;
        }
    };

    let mut start_perc = get_bckl_perc(&backlight).await.unwrap();
    let mut interval = time::interval(Duration::from_millis(RESOLUTION * 2));

    loop {
        interval.tick().await;
        let cur_perc = get_bckl_perc(&backlight).await.unwrap();
        if cur_perc != start_perc {
            let mut context = HashMap::new();
            context.insert("old_backlight".to_string(), format!("{start_perc}"));
            context.insert("new_backlight".to_string(), format!("{cur_perc}"));
            // println!("returning context =>  {:#?}", context);
            match return_tx.send(context) {
                Ok(_) => {}
                Err(e) => println!("{e}\n[ERROR] could not send backlight context."),
            };
            start_perc = cur_perc;
        }
    }
}

async fn make_usb_context(new_devs: &[UsbDevice]) -> Context {
    let mut new_dev_names = Vec::new();
    let mut new_dev_id = Vec::new();
    let mut context = HashMap::new();

    for dev in new_devs {
        match &dev.description {
            Some(name) => {
                new_dev_names.push(name.clone());
                new_dev_id.push(dev.id.clone());
            }
            None => {
                new_dev_names.push(String::new());
                new_dev_id.push(dev.id.clone());
            }
        };
    }

    context.insert("device_names".to_string(), new_dev_names.join(","));
    context.insert("device_ids".to_string(), new_dev_id.join(","));

    context
}

pub async fn new_usb(return_tx: UnboundedSender<Context>) {
    // println!("new usb");
    let mut interval = time::interval(Duration::from_millis(RESOLUTION * 2));
    let devices = usb_enumeration::enumerate(None, None).into_iter().collect::<HashSet<UsbDevice>>();

    // println!("{:?}", devices.len());
    loop {
        interval.tick().await;
        let tmp_devices = usb_enumeration::enumerate(None, None).into_iter().collect::<HashSet<UsbDevice>>();
        if tmp_devices != devices {
            let new_devices = tmp_devices.into_iter().filter(|dev| !devices.contains(dev));
            match return_tx.send(make_usb_context(&new_devices.collect::<Vec<UsbDevice>>()).await) {
                Ok(_) => {}
                Err(e) => {
                    println!("{e}\n[ERROR] could not send usb context.")
                }
            };
        }
    }
}

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().next().unwrap()
}

/// returns and event stream.
async fn get_blt_con(connected: &mut HashSet<String>) -> Result<Context, Box<dyn Error>> {
    let manager = Manager::new().await?;
    // get the first bluetooth adapter then connect
    let central = get_central(&manager).await;

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;
    // Ok(events)
    let mut context = HashMap::new();
    let mut adr = String::new();

    while let Some(event) = events.next().await {
        // println!("event {:?}", event);
        match event {
            CentralEvent::DeviceConnected(id) => {
                let tmp_adr = make_adr(&format!("{}", id)).await;
                // println!("connected devices at if => {:?}", connected);
                if !connected.contains(&tmp_adr) {
                    // println!("[DEBUG] bluetooth device connected: {:?}", id);
                    connected.insert(tmp_adr.clone());
                    context.insert("event".to_string(), "connected".to_string());
                    adr = tmp_adr;
                    break;
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                // println!("[DEBUG] bluetooth device disconnected: {:?}", id);
                let tmp_adr = make_adr(&format!("{}", id)).await;
                if connected.contains(&tmp_adr) {
                    connected.remove(&tmp_adr);
                }
                adr = tmp_adr;
                context.insert("event".to_string(), "disconnected".to_string());
                break;
            }
            _ => {}
        }
    }
    context.insert("device_adr".to_string(), adr);
    Ok(context)
}

async fn make_adr(obj_path: &str) -> String {
    // println!("obj_path => {}", obj_path);
    match obj_path.split_once("dev_") {
        Some((_, underbar_adr)) => {
            underbar_adr.replace(0 as char, "").replace('_', ":")
        }
        None => panic!("this should not be reachable. if it was there was a problem when parsing the address of a bluetooth device.")
    }
}

pub async fn blt_dev_conn(return_tx: UnboundedSender<Context>) {
    // println!("bluetooth dev conn");
    // let mut connected = old_connected.clone();
    let mut interval = time::interval(Duration::from_millis(RESOLUTION * 3));
    let mut connected: HashSet<String> = HashSet::new();
    match get_blt_con(&mut connected).await {
        Ok(_) => {}  // context.keys().collect(),
        Err(e) => {
            println!("{e}\n[ERROR] could not get connected bluetooth devices. is the adapter plugged in and powered on?");
            return;
        }
    };

    let mut default_context = HashMap::new();
    default_context.insert("event".to_string(), "N/A".to_string());
    default_context.insert("device_adr".to_string(), "N/A".to_string());
    // println!("connected devices before => {:?}", connected);
    loop {
        interval.tick().await;
        // let old_dev_list = connected.clone();
        let devs = match get_blt_con(&mut connected).await {
            Ok(context) => context,
            Err(_) => default_context.clone(), 
        };
        if !connected.is_empty() {
            match return_tx.send(devs) {
                Ok(_) => {}
                Err(e) => println!("{e}\n[ERROR] could not send bluetooth device connection data.")
            }

        }
    }
}

fn make_context(port: Port) -> Context {
    // println!("exec => {:?}:{:?} {:?}", port.exec, port.lport, port.rport);
    let mut context = HashMap::new();
    context.insert("local_adr".to_string(), port.local_addr);
    context.insert("remote_adr".to_string(), port.remote_addr);
    context.insert("state".to_string(), port.state);
    context.insert("executable".to_string(), match port.exec {
        Some(exe) => exe,
        None => String::new(),
    });
    // context.insert("inode".to_string(), format!("{}", port.inode));
    context.insert("pid".to_string(), match port.pid {
        Some(pid) => format!("{}", pid),
        None => "".to_string(),
    });
    // context.insert("became".to_string(), became.to_string());
    // context.insert("port".to_string(), port.port.to_string());
    context.insert("connection_direction".to_string(), port.con_dir.to_string());
    context.insert("local_port".to_string(), port.lport.to_string());
    context.insert("remote_port".to_string(), port.rport.to_string());

    context
}

async fn make_port_context(ports: Vec<Port>) -> Vec<Context> {
    // let contexts: Vec<Context> = Vec::new();

    // for port in ports {
    //     contexts.push(make_context(port));
    // }

    // contexts
    ports.into_iter().map(make_context).collect()
}

async fn make_port(port_dat: &[&str], stop_execs: &HashSet<String>) -> Option<Port> {
    // 0     ,      1         ,      2           ,      3          ,      4            ,       5                    
    // {pid}{DELIM}{local ip}{DELIM}{local port}{DELIM}{remote_ip}{DELIM}{remote port}{DELIM}{INCOMING/OUT-GOING}
    let tmp_pid = port_dat[0];
    
    // get the executable associated with the provided pid.
    let (proc_id, executable) = match tmp_pid.parse() {
        Ok(pid) => {
            match Process::new(pid) {
                Ok(proc) => {
                    match proc.exe().unwrap_or(PathBuf::new()).file_name() {
                        Some(path) => {
                            let exe = path.to_str().unwrap_or("").to_string();
                            (Some(pid), Some(exe))
                        },
                        None => (Some(pid), None),
                    }
                },
                Err(_e) => (Some(pid), None), 
            } 
        }
        Err(_e) => {
            println!("[ERROR] could not interpret pid, '{}', as an i32", port_dat[0]);
            (None, None)
        }
    };

    // println!("port shift => {:?}:{:?}", exec, pid);

    let port = Some(
        Port{ 
            local_addr: port_dat[1].to_string(),
            remote_addr: port_dat[3].to_string(),
            lport: port_dat[2].to_string(),
            rport: port_dat[4].to_string(),
            state: "".to_string(),
            pid: proc_id,
            exec: executable.clone(),
            // port: port_dat[2].to_string(),
            con_dir: port_dat[5].to_string(),
        }
    );

    match executable {  
        Some(exec) if !stop_execs.contains(&exec) => port,
        None => port,
        _ => None,
    }
}

pub async fn port_change(stop_execs: HashSet<String>, ignore_web: bool, return_tx: UnboundedSender<Vec<Context>>) {
    let pipe_f = config::get_pipe_f();

    let listener = match UnixListener::bind(&pipe_f) {
        Ok(listener) => listener,
        Err(e) => {
            println!("[ERROR] could not generate ports listener at \"{pipe_f}\" because of error, '{e}'.");
            println!("[LOG] ports event hook disabled.");
            return;
        }
    };
    

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let mut ports = String::new();
                
                match stream.read_to_string(&mut ports).await {
                    Ok(_) => {},
                    Err(e) => eprintln!("[ERROR] failed to read from port socket: {e}"),
                }
                if let Err(reason) = stream.shutdown().await {
                    println!("[ERROR] could not shut down input stream because, \"{reason}\"");
                }


                let mut parsed_ports = Vec::new();

                for line in ports.split('\n') {
                    let port_dat: Vec<&str> = line.split(DELIM).collect();  
                    // {error-code}{DELIM}{pid}{DELIM}{local ip}{DELIM}{local port}{DELIM}{remote_ip}{DELIM}{remote port}{DELIM}{INCOMING/OUT-GOING/LOCAL}{DELIM}{index}
                    // eprint!("UID {} => ", port_dat[7]);
                    if line.as_bytes()[0] as char != ERROR {
                        if let Some(port) = make_port(&port_dat[1..], &stop_execs).await {
                            // println!("{:?}", port);
                            if (port.con_dir.to_lowercase() != "out-going" || !["80", "443"].contains(&port.rport.as_str())) && ignore_web {
                                parsed_ports.push(port);
                            }
                        } 
                        // else {
                        //     println!("{:?}", port_dat);
                        // }
                    } else {
                        eprintln!("[ERROR] {}", port_dat[1]);
                    }
                }

                let contexts = make_port_context(parsed_ports.clone().into_iter().filter(|port| !stop_execs.contains(&port.exec.clone().unwrap_or(String::new()))).collect::<Vec<Port>>()).await;

                if !contexts.is_empty() { 
                    // println!("context => {:#?}", contexts);
                    // println!("ports => {:#?}", ports);
                    // println!("inodes => {:#?}", parsed_ports.keys());

                    if let Err(e) = return_tx.send(contexts) {
                        println!("[ERROR] could not send port context to main event process. got  error:\n{e}");
                    }
                    // println!("sent");
                }
            },
            Err(e) => {
                println!("could not read from data from unix socket at {pipe_f}. got error {e}");
            }
        }
        sleep(Duration::from_millis(100)).await;
    }
    // println!("end of function")
}
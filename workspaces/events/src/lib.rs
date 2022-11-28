use procfs::process::{FDTarget, Stat};
use tokio::time::{sleep, Duration};
// use tokio::sync::mpsc::{Sender, Receiver, channel};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use usb_enumeration::UsbDevice;
use std::collections::HashMap;
use std::collections::HashSet;
use online::tokio::check;
use tokio::fs;
use tokio::time;
// use usb_enumeration;
use btleplug::api::{Central, CentralEvent, Manager as _,};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use iw::interfaces;
// use notify::{Event, RecommendedWatcher, PollWatcher, RecursiveMode, Watcher, Config};
// use futures::{
//     channel::mpsc::{channel, Receiver},
//     SinkExt, StreamExt,
// };

pub type Context = HashMap<String, String>;
const RESOLUTION: u64 = 1000;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Port {
    local_addr: String, 
    remote_addr: String, 
    state: String, 
    inode: u64, 
    pid: Option<i32>, 
    exec: Option<String>,
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

pub async fn network_connection() -> Context {
    // println!("net connected");

    let connected = check(None).await.is_ok();
    let mut context = HashMap::new();
    loop {
        sleep(Duration::from_millis(RESOLUTION * 2)).await;
        let tmp_connected = check(None).await.is_ok();
        if tmp_connected != connected {
            context.insert(
                "became".to_string(), 
                if tmp_connected {"connected".to_string()} else {"disconnected".to_string()}
            );
            return context;
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

pub async fn wifi_change() -> Context {
    // println!("wifi change");
    let mut ssid = get_name();
    loop {
        sleep(Duration::from_millis(RESOLUTION)).await;
        let tmp_ssid = get_name();
        if tmp_ssid != ssid {
            let mut context = HashMap::new();
            context.insert("old_network".to_string(), ssid);
            context.insert("new_network".to_string(), tmp_ssid);
            return context;
        } else {
            ssid = tmp_ssid;
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

pub async fn backlight_change() -> Context {
    // println!("backlight change");

    let backlight = match tokio::fs::read_dir("/sys/class/backlight/").await {
        Ok(mut dirs) => {
            match dirs.next_entry().await {
                Ok(Some(dir)) => dir,
                _ => return HashMap::new()
            }
        }
        Err(_) => {
            println!("could not read \"/sys/class/backlight\" directory.");
            return HashMap::new();
        }
    };

    let start_perc = get_bckl_perc(&backlight).await.unwrap();
    let mut interval = time::interval(Duration::from_millis(RESOLUTION));

    loop {
        interval.tick().await;
        let cur_perc = get_bckl_perc(&backlight).await.unwrap();
        if cur_perc != start_perc {
            let mut context = HashMap::new();
            context.insert("old_backlight".to_string(), format!("{start_perc}"));
            context.insert("new_backlight".to_string(), format!("{cur_perc}"));
            // println!("returning context =>  {:#?}", context);
            return context;
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

pub async fn new_usb() -> Context {
    // println!("new usb");
    let mut interval = time::interval(Duration::from_millis(RESOLUTION));
    let devices = usb_enumeration::enumerate(None, None).into_iter().collect::<HashSet<UsbDevice>>();

    // println!("{:?}", devices.len());
    loop {
        interval.tick().await;
        let tmp_devices = usb_enumeration::enumerate(None, None).into_iter().collect::<HashSet<UsbDevice>>();
        if tmp_devices != devices {
            let new_devices = tmp_devices.into_iter().filter(|dev| !devices.contains(dev));
            return make_usb_context(&new_devices.collect::<Vec<UsbDevice>>()).await;
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

pub async fn blt_dev_conn(mut connected: HashSet<String>) -> (Context, HashSet<String>) {
    // println!("bluetooth dev conn");
    // let mut connected = old_connected.clone();
    let mut default_context = HashMap::new();
    default_context.insert("event".to_string(), "N/A".to_string());
    default_context.insert("device_adr".to_string(), "N/A".to_string());
    // println!("connected devices before => {:?}", connected);
    (match get_blt_con(&mut connected).await {
        Ok(context) => context,
        Err(_) => default_context, 
    }, connected)
}

/// returns open tcp ports
async fn get_tcp_ports(stop_execs: &HashSet<String>) -> HashSet<Port> {
    // get all processes
    let all_procs = procfs::process::all_processes().unwrap();

    // build up a map between socket inodes and processes:
    let mut map: HashMap<u64, Stat> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd_info in fds.flatten() {
                    if let FDTarget::Socket(inode) = fd_info.target {map.insert(inode, stat.clone());}
            }
        }
    }

    let tcp = procfs::net::tcp().unwrap();
    let tcp6 = procfs::net::tcp6().unwrap();
    // this way it ignores ports having to do with web browsers
    let mut ports = HashSet::new();

    for entry in tcp.into_iter().chain(tcp6) {
        // find the process (if any) that has an open FD to this entry's inode
        let local_address = format!("{}", entry.local_address);
        let remote_address = format!("{}", entry.remote_address);
        let port_state = format!("{:?}", entry.state);
        if let Some(stat) = map.get(&entry.inode) {
            // println!(
            //     "{:<26} {:<26} {:<15} {:<12} {}/{}",
            //     local_address, remote_addr, state, entry.inode, stat.pid, stat.comm
            // );

            // this way it ignores ports having to do with web browsers
            if !stop_execs.contains(&stat.comm) {
                let port = Port {
                    local_addr: local_address, 
                    remote_addr: remote_address, 
                    state: port_state, 
                    inode: entry.inode, 
                    pid: Some(stat.pid), 
                    exec: Some(stat.comm.clone()),
                };
                // println!("{:?}", port);
                ports.insert(port);
            }
        } else {
            // We might not always be able to find the process assocated with this socket
            // println!(
            //     "{:<26} {:<26} {:<15} {:<12} -",
            //     local_address, remote_addr, state, entry.inode
            // );
            // let port = Port {
            //     local_addr: local_address, 
            //     remote_addr: remote_addr, 
            //     state: state, 
            //     // inode: entry.inode, 
            //     pid: None, 
            //     exec: None,
            // };
            // ports.insert(port);
        }
    }

    ports
}

async fn get_changed(old_ports: HashSet<Port>, new_ports: HashSet<Port>) -> (HashSet<Port>, HashSet<Port>) {
    let old_diff = old_ports.difference(&new_ports).into_iter().map(|p| p.to_owned()).collect::<HashSet<Port>>();
    let new_diff = new_ports.difference(&old_ports).into_iter().map(|p| p.to_owned()).collect::<HashSet<Port>>();
    (old_diff, new_diff)
}

fn make_context(closed_port: Port, became: &str) -> Context {
    let mut context = HashMap::new();
    context.insert("local_adr".to_string(), closed_port.local_addr);
    context.insert("remote_adr".to_string(), closed_port.remote_addr);
    context.insert("state".to_string(), closed_port.state);
    context.insert("executable".to_string(), match closed_port.exec {
        Some(exe) => exe,
        None => "UNKNOWN".to_string(),
    });
    context.insert("inode".to_string(), format!("{}", closed_port.inode));
    context.insert("l_addr".to_string(), match closed_port.pid {
        Some(pid) => format!("{}", pid),
        None => "UNKNOWN".to_string(),
    });
    context.insert("became".to_string(), became.to_string());

    context
}

fn make_port_contexts(closed: HashSet<Port>, opened: HashSet<Port>) -> Vec<Context> {
    let mut contexts = Vec::new();
    // let mut ;
    for closed_port in closed {
        contexts.push(make_context(closed_port, "closed"));
    }

    for opened_port in opened {
        contexts.push(make_context(opened_port, "opened"));
    }

    contexts
}

async fn get_tcp_conn(stop_execs: HashSet<String>, sender: UnboundedSender<(HashSet<Port>, HashSet<Port>)>) {
    let mut open_ports = get_tcp_ports(&stop_execs).await;

    loop {
        let new_open_ports = get_tcp_ports(&stop_execs).await;

        if open_ports != new_open_ports {
            let _ = sender.send(get_changed(open_ports, new_open_ports).await);
            break;
            // let (closed, opened) = get_changed(open_ports, new_open_ports).await;
            // return make_port_contexts(closed, opened).await;
        } else {
            open_ports = new_open_ports;
        }
    }
}

pub async fn port_change(stop_execs: &HashSet<String>) -> Vec<Context> {
    // println!("port state change");

    // let mut open_ports = get_tcp_ports(stop_execs).await;
    let mut interval = time::interval(Duration::from_millis(RESOLUTION));
    let (tx, mut rx) = unbounded_channel::<(HashSet<Port>, HashSet<Port>)>();
    let corout = tokio::task::spawn(get_tcp_conn(stop_execs.clone(), tx));

    loop {
        
        tokio::select! {
            _ = interval.tick() => {},
           res = rx.recv() => {
                match res {
                    Some((closed, opened)) => {
                        corout.abort();
                        return make_port_contexts(closed, opened);
                    }
                    None => {}
                }
                
            },
        }
    }
}
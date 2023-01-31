use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use crate::wm_lib;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::time::{sleep, Duration};
use crate::common;
use crate::config::OptGenRes;

const NULL: char = 0 as char;

type WMClass = String;
type Desktop = String;
type Desktops = HashMap<Desktop, bool>;
type Exe = String;
type Programs = Vec<Exe>;
type Rules = HashMap<WMClass, Vec<Desktop>>;


pub enum QtileAPI {
    Layout(QtileCmdData),
    Message(String),
    Res(u8),
}

// #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QtileCmdData {
    pub rules: Rules,
    pub queue: Programs,
    pub clear: Desktops,
}

impl QtileCmdData {
    fn new() -> QtileCmdData {
        QtileCmdData {
            rules: HashMap::new(),
            queue: Vec::new(),
            clear: HashMap::new(),
        }
    }

    fn add_clear(&mut self, to_clear: Option<bool>, data: &str) {
        let clear = match to_clear {
            Some(b) => b,
            _ => false,
        };
        self.clear.insert(data.to_string(), clear);
    }

    fn add_queue(&mut self, program: &str, args: &Option<Vec<String>>) {
        self.queue.push(
            match args {
                Some(args) => format!("{program} {}", args.join(" ")),
                None => program.to_string()
            }
        );
    }

    fn add_rules(&mut self, wmc: &str, desktop: &str) {
        match self.rules.get_mut(wmc) {
            Some(rules) => {rules.push(desktop.to_string());},
            None => {
                let mut set = Vec::new();
                set.push(desktop.to_string());
                self.rules.insert(wmc.to_string(), set);
            }
        };
    }

    fn get_location_helper(&mut self, wm_class: &str) -> Option<String> {
        // println!("wm_class :  {wm_class}, rules :  {:?}", self.rules);
        match self.rules.get_mut(wm_class) {
            Some(desktops) => desktops.pop(),
            None => None
        }
    }

    fn get_location(&mut self, wm_classes: (&str, &str)) -> Option<String> {
        match self.get_location_helper(wm_classes.0) {
            Some(location) => Some(location),
            None =>  self.get_location_helper(wm_classes.1),
        }
    }

    fn should_clear(&mut self, group: &str) -> bool {
        match self.clear.remove(group) {
            Some(b) => b,
            None => false,
        }
    }
}

// pub fn move_to(_spath: &str, _args: &str) -> u8 {
//     0
// }

// pub fn close_focused(spath: &str) -> u8 {
//     0
// }

pub fn should_clear(args: &str, layout: &mut Option<QtileCmdData>) -> Result<bool, u8> {
    let tmp_layout = match layout {
        Some(lo) => lo,
        None => return Ok(false),
    };

    let arguments: Vec<&str> = args.split_ascii_whitespace().collect();
    let location = if arguments.len() == 1 {
        arguments[0]
    } else {
        return Err(7);
    };

    let clearing = tmp_layout.should_clear(location);
    Ok(clearing)
}

pub fn auto_move(args: &str, layout: &mut Option<QtileCmdData>) -> Result<Option<String>, u8> {
    // println!("auto-move");
    // println!("args => {}", args);
    // println!("layout => {:?}", layout);
    let tmp_layout = match layout {
        Some(lo) => lo,
        None => {
            println!("[DEBUG] no layout provided.");
            return Ok(None)
        },
    };
    let arguments = args.splitn(2," ").collect::<Vec<&str>>();
    if arguments.len() != 2 {
        println!("[ERROR] wrong number of arguments, {}", arguments.len());
        return Err(7);
    }

    let wm_classes = (arguments[0], arguments[1]);
    println!("[DEBUG] wm_classes: {:?}", wm_classes);

    let location = match tmp_layout.get_location(wm_classes) {
        Some(location) => location,
        None => {
            println!("[DEBUG] wm_class not in layout.");
            return Ok(None)
        }
    };

    println!("[DEBUG] moving window with class: {:?} to location: {location}, will be handled by qtile.", wm_classes);
    Ok(Some(location))
}

/// open-at
pub fn open_on_desktop(spath: &str, args: &str) -> u8 {
    let data = args.split(" ").collect::<Vec<&str>>();
    
    if data.len() != 3 {
        return 7
    }

    let (exe, wm_class, desktop) = (data[0], data[1], data[2]);
    let payload = format!("load{NULL}{{\"rules\": {{ \"{wm_class}\": [\"{desktop}\"]}}, \"queue\": [\"{exe}\"] }}");

    println!("payload => {}", payload);

    let load_res = qtile_send(payload, spath);
    // thread::sleep(time::Duration::from_millis(20));
    let clear_res = qtile_send("clear".to_string(), spath);
    common::open_program(exe);
    if clear_res > 0 {
        println!("failed to clear, got error: {}", clear_res);
        3
    } else if load_res > 0 {
        load_res
    } else {
        0
    }
    // load_res
}

/// focus-on
pub fn focus_on(spath: &str, args: &str) -> u8 {
    println!("focus_on");
    println!("spath => {}", spath);
    println!("args => {}", args);
    0
}

pub fn make_cmd_data(fname: &str) -> Result<QtileCmdData, u8> {
    let layouts = match wm_lib::get_layout(fname) {
        Ok(layout) => layout,
        Err(n) => return Err(n),
    };

    let mut payload_struc = QtileCmdData::new();        
    
    for desktop in layouts {
        for program in &desktop.programs {
            match &program.wm_class {
                Some(class) => {
                    payload_struc.add_rules(&class, &desktop.desktop);
                    payload_struc.add_queue(&program.name, &program.args);
                }
                None => println!("no wm_class defined for {} in the layout file. could not setup or launch.", program.name),
            }
        }
        payload_struc.add_clear(desktop.clear, &desktop.desktop);
    }

    Ok(payload_struc)
}

/// load-layout
pub async fn load_layout(spath: &str, args: &str) -> u8 {
    // println!("load_layout");
    // println!("spath => {}", spath);
    // println!("args => {}", args);

    let (payload, programs) = match make_payload(args) {
        Ok(payload) => payload,
        Err(ec) => return ec, 
    };

    // println!("payload => {}", payload);
    
    let load_res = qtile_send(payload, spath);
    // thread::sleep(time::Duration::from_millis(20));
    let clear_res = qtile_send("clear".to_string(), spath);

    sleep(Duration::from_millis(500)).await;  // TODO: try commenting this out and/or lowering the time.
    
    if load_res == 0 {
        for program in programs {
            common::open_program(&program);
        }
    }
    
    if clear_res > 0 {
        println!("failed to clear, got error: {}", clear_res);
        3
    } else if load_res > 0 {
        load_res
    } else {
        0
    }
}

fn make_payload(fname: &str) -> Result<(String, Vec<String>), u8>  {
    let payload_struc = make_cmd_data(fname)?;

    match serde_json::to_string(&payload_struc) {
        Ok(jsons) => Ok((jsons, payload_struc.queue)),
        Err(e) => {
            println!("[DEBUG] got error while serializing to qtile-data to json.");
            println!("error: {}", e);
            Err(4)
        }
    }
}

fn qtile_send(payload: String, spath: &str) -> u8 {
    let mut stream = match UnixStream::connect(spath) {
        Ok(stream) => stream,
        Err(_) => return 5,
    };

    let _ = stream.write_all(&payload.into_bytes());

    match stream.shutdown(Shutdown::Write) {
        Ok(_) => {}
        Err(e) => {
            println!("[ERROR] :  failed to shutdown write access to socket file.");
            println!("[DEBUG] :  {}", e);
            return 5;
        }
    };

    let mut response_bytes = Vec::new();
    match stream.read_to_end(&mut response_bytes) {
        Ok(_) => {}
        Err(e) => {
            println!("could not read response from server.");
            println!("[DEBUG] :  {}", e);
            return 2;
        }
    };

    if response_bytes.len() != 0  {
        let (ec, res) = (response_bytes[0], &response_bytes[1..]);
        match std::str::from_utf8(res) {
            Ok(res) => {
                println!("[LOG] qtile responded: {}", res);
                ec
            }
            Err(e) => {
                println!("{}", e);
                5
            }
        }
    }
    else {
        println!("[ERROR] The qtile python api didn't respond. It most likely crashed. Pls reload it and try again.");
        6
    }    
}

pub async fn qtile_switch(cmd: &str, args: &str, spath: &str) -> OptGenRes {
    match cmd {
        // "move-to" => Some(move_to(spath, args)),
        // "close-focused" => Some(close_focused(spath)),
        "open-at" | "open-on" => Some((open_on_desktop(spath, args), None)),
        "focus-on" => Some((focus_on(spath, args), None)),
        _ => None,
    }
}

pub async fn qtile_api(
    cmd: &str, 
    args: &str, 
    layout: &mut Option<QtileCmdData>
) -> Option<QtileAPI> {
    match cmd {
        "load-layout" => {
            match make_cmd_data(args) {
                Ok(layout) => Some(QtileAPI::Layout(layout)),
                Err(ec) => Some(QtileAPI::Res(ec)),
            }
        }
        "auto-move" => Some(
            match auto_move(args, layout) {
                Ok(Some(loc)) => QtileAPI::Message(loc),
                Ok(None) => QtileAPI::Res(0), 
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        "should-clear" => Some(
            match should_clear(args, layout) {
                Ok(to_clear_or_not_to_clear) => QtileAPI::Message(if to_clear_or_not_to_clear {"true"} else {"false"}.to_string()), // that is the question
                Err(ec) => QtileAPI::Res(ec),
            }
        ),
        _ => None,
    }
}
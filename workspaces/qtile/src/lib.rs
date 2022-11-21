// use std::{thread, time};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use wm_lib;
use wm_lib::DesktopLayout;
use serde::{Deserialize, Serialize};
use serde_json;
use common;

const NULL: char = 0 as char;

type WMClass = String;
type Desktop = String;
type Desktops = HashMap<Desktop, bool>;
type Exe = String;
type Programs = HashSet<Exe>;
type Rules = HashMap<WMClass, HashSet<Desktop>>;

// #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct QtileCmdData {
    rules: Rules,
    queue: Programs,
    clear: Desktops,
}

impl QtileCmdData {
    fn new() -> QtileCmdData {
        QtileCmdData {
            rules: HashMap::new(),
            queue: HashSet::new(),
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

    fn add_queue(&mut self, program: &str) {
        self.queue.insert(program.to_string());
    }

    fn add_rules(&mut self, wmc: &str, desktop: &str) {
        match self.rules.get_mut(wmc) {
            Some(rules) => {rules.insert(desktop.to_string());},
            None => {
                let mut set = HashSet::new();
                set.insert(desktop.to_string());
                self.rules.insert(wmc.to_string(), set);
            }
        };
    }
}

// pub fn move_to(_spath: &str, _args: &str) -> u8 {
//     0
// }

// pub fn close_focused(spath: &str) -> u8 {
//     0
// }

/// open-at
pub fn open_on_desktop(spath: &str, args: &str) -> u8 {
    println!("open_on_desktop");
    // println!("spath => {}", spath);
    // println!("args => {}", args);

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

/// load-layout
pub fn load_layout(spath: &str, args: &str) -> u8 {
    println!("load_layout");
    println!("spath => {}", spath);
    println!("args => {}", args);
    
    let layout_yaml = match wm_lib::get_layout(args) {
        Ok(layout) => layout,
        Err(n) => return n,
    };

    let payload = match make_payload(&layout_yaml) {
        Ok(payload) => format!("load{NULL}{}", payload),
        Err(_) => return 2,
    };

    println!("payload => {}", payload);
    
    let load_res = qtile_send(payload, spath);
    // thread::sleep(time::Duration::from_millis(20));
    let clear_res = qtile_send("clear".to_string(), spath);
    
    if load_res == 0 {
        for desktop in layout_yaml {
            for program in desktop.programs {
                common::open_program(&program.name);
            }
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
    // load_res
}

fn make_payload(layouts: &Vec<DesktopLayout>) -> Result<String, ()>  {
    let mut payload_struc = QtileCmdData::new();        
    
    for desktop in layouts {
        for program in &desktop.programs {
            match &program.wm_class {
                Some(class) => {
                    payload_struc.add_rules(&class, &desktop.desktop);
                    payload_struc.add_queue(&program.name);
                }
                None => println!("no wm_class defined for {} in the layout file. could not setup or launch.", program.name),
            }
        }
        payload_struc.add_clear(desktop.clear, &desktop.desktop);
    }

    match serde_json::to_string(&payload_struc) {
        Ok(jsons) => Ok(jsons),
        Err(e) => {
            println!("[DEBUG] got error while serializing to qtile-data to json.");
            println!("error: {}", e);
            Err(())
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
use crate::common;
use crate::config::OptGenRes;
use crate::wm_lib;
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn new() -> QtileCmdData {
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
        self.queue.push(match args {
            Some(args) => format!("{program} {}", args.join(" ")),
            None => program.to_string(),
        });
    }

    fn add_rules(&mut self, wmc: &str, desktop: &str) {
        match self.rules.get_mut(wmc) {
            Some(rules) => {
                rules.push(desktop.to_string());
            }
            None => {
                let set = vec![desktop.to_string()];
                self.rules.insert(wmc.to_string(), set);
            }
        };
    }

    fn get_location_helper(&mut self, wm_class: &str) -> Option<String> {
        match self.rules.get_mut(wm_class) {
            Some(desktops) => desktops.pop(),
            None => None,
        }
    }

    fn get_location(&mut self, wm_classes: (&str, &str)) -> Option<String> {
        match self.get_location_helper(wm_classes.0) {
            Some(location) => Some(location),
            None => self.get_location_helper(wm_classes.1),
        }
    }

    fn should_clear(&mut self, group: &str) -> bool {
        self.clear.remove(group).unwrap_or(false)
    }
}

// pub fn move_to(_spath: &str, _args: &str) -> u8 {
//     0
// }

// pub fn close_focused(spath: &str) -> u8 {
//     0
// }

pub fn should_clear(args: &str, layout: &mut QtileCmdData) -> Result<bool, u8> {
    // let tmp_layout = match layout {
    //     Some(lo) => lo,
    //     None => return Ok(false),
    // };

    let arguments: Vec<&str> = args.split_ascii_whitespace().collect();
    let location = if arguments.len() == 1 {
        arguments[0]
    } else {
        return Err(7);
    };

    let clearing = layout.should_clear(location);
    Ok(clearing)
}

pub fn auto_move(args: &str, layout: &mut QtileCmdData) -> Result<Option<String>, u8> {
    // debug!("auto-move");
    // debug!("args => {}", args);
    // debug!("layout => {:?}", layout);

    let arguments = args.splitn(2, ' ').collect::<Vec<&str>>();
    if arguments.len() != 2 {
        error!("wrong number of arguments, {}", arguments.len());
        return Err(7);
    }

    let wm_classes = (arguments[0], arguments[1]);
    debug!("wm_classes: {:?}", wm_classes);

    let location = match layout.get_location(wm_classes) {
        Some(location) => location,
        None => {
            debug!("wm_class not in layout.");
            return Ok(None);
        }
    };

    debug!(
        "moving window with class: {wm_classes:?} to location: {location}, will be handled by qtile."
    );
    Ok(Some(location))
}

/// open-at
pub fn open_on_desktop(_spath: &str, args: &str, layout: &mut QtileCmdData) -> u8 {
    let data = args.split(' ').collect::<Vec<&str>>();

    if data.len() != 3 {
        return 7;
    }

    let (exe, wm_class, desktop) = (data[0], data[1], data[2]);

    layout.add_rules(wm_class, desktop);

    common::open_program(exe)
}

/// focus-on
pub fn focus_on(spath: &str, args: &str) -> u8 {
    trace!("focus_on");
    debug!("spath => {}", spath);
    debug!("args => {}", args);
    // TODO: write this
    0
}

pub fn make_cmd_data(fname: &str) -> Result<QtileCmdData, u8> {
    let layouts = match wm_lib::get_layout(fname) {
        Ok(layout) => layout,
        Err(n) => return Err(n),
    };

    let mut payload_struct = QtileCmdData::new();

    for desktop in layouts.desktops {
        for program in &desktop.programs {
            match &program.wm_class {
                Some(class) => {
                    payload_struct.add_rules(class, &desktop.desktop);
                    payload_struct.add_queue(&program.name, &program.args);
                }
                None => error!(
                    "no wm_class defined for {} in the layout file. could not setup or launch.",
                    program.name
                ),
            }
        }
        payload_struct.add_clear(desktop.clear, &desktop.desktop);
    }

    Ok(payload_struct)
}

/// load-layout
pub async fn load_layout(spath: &str, args: &str) -> u8 {
    trace!("load_layout");
    debug!("spath => {}", spath);
    debug!("args => {}", args);

    let (_payload, programs) = match make_payload(args) {
        Ok(payload) => payload,
        Err(ec) => return ec,
    };

    for program in programs {
        let res = common::open_program(&program);

        if res > 0 {
            error!("launching \"{program}\", returned a non-zero error-code.");
        }
    }

    // TODO: set workspaces to desktops
    0
}

fn make_payload(fname: &str) -> Result<(String, Vec<String>), u8> {
    let payload_struct = make_cmd_data(fname)?;

    match serde_json::to_string(&payload_struct) {
        Ok(jsons) => Ok((jsons, payload_struct.queue)),
        Err(e) => {
            error!("got error while serializing to qtile-data to json. error:\"{e}\"");
            Err(4)
        }
    }
}

pub async fn qtile_switch(
    cmd: &str,
    args: &str,
    spath: &str,
    layout: &mut QtileCmdData,
) -> OptGenRes {
    match cmd {
        // "move-to" => Some(move_to(spath, args)),
        // "close-focused" => Some(close_focused(spath)),
        "open-at" | "open-on" => Some((open_on_desktop(spath, args, layout), None)),
        "focus-on" => Some((focus_on(spath, args), None)),
        _ => None,
    }
}

pub async fn qtile_api(cmd: &str, args: &str, layout: &mut QtileCmdData) -> Option<QtileAPI> {
    match cmd {
        "load-layout" => match make_cmd_data(args) {
            Ok(layout) => Some(QtileAPI::Layout(layout)),
            Err(ec) => Some(QtileAPI::Res(ec)),
        },
        "auto-move" => Some(match auto_move(args, layout) {
            Ok(Some(loc)) => QtileAPI::Message(loc),
            Ok(None) => QtileAPI::Res(0),
            Err(ec) => QtileAPI::Res(ec),
        }),
        "should-clear" => Some(match should_clear(args, layout) {
            Ok(to_clear_or_not_to_clear) => QtileAPI::Message(
                if to_clear_or_not_to_clear {
                    "true"
                } else {
                    "false"
                }
                .to_string(),
            ), // that is the question
            Err(ec) => QtileAPI::Res(ec),
        }),
        _ => None,
    }
}

use crate::config;
use crate::config::Hook;
use crate::config::OptGenRes;
use crate::events;
use crate::msgs;
use events::Context;
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use tokio::process::Command;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;

pub type HookID = u16;
pub type Hooks = HashMap<HookID, Hook>;

#[derive(Clone, Debug)]
pub struct HookDB {
    pub hooks: Hooks,
    pub next_uid: HookID,
    pub wifi_net: Vec<HookID>,
    pub net_con: Vec<HookID>,
    pub backlight: Vec<HookID>,
    pub usb_dev: Vec<HookID>,
    // pub bluetooth_dev: Vec<HookID>,
    pub bluetooth_conn: Vec<HookID>,
    pub ports_change: Vec<HookID>,
    // pub test_file_exists: Vec<HookID>,
}

impl Default for HookDB {
    fn default() -> Self {
        Self::new()
    }
}

impl HookDB {
    pub fn new() -> HookDB {
        HookDB {
            hooks: HashMap::new(),
            next_uid: 0,
            wifi_net: Vec::new(),
            net_con: Vec::new(),
            backlight: Vec::new(),
            usb_dev: Vec::new(),
            // bluetooth_dev: Vec::new(),
            bluetooth_conn: Vec::new(),
            ports_change: Vec::new(),
            // test_file_exists: Vec::new(),
        }
    }

    fn add_hook(&mut self, hook: config::Hook) -> Result<(), ()> {
        let hook_id = self.next_uid;
        // TODO: make this use an enum, not static str. (why isn't this already the case)
        match hook.event.as_str() {
            "wifi-network-change" => self.wifi_net.push(hook_id),
            "network-toggle" => self.net_con.push(hook_id),
            "backlight" => self.backlight.push(hook_id),
            "new-usb-device" => self.usb_dev.push(hook_id),
            "bluetooth-device" => self.bluetooth_conn.push(hook_id),
            "port-status-change" => self.ports_change.push(hook_id),
            // "test_file_exists" => self.test_file_exists.push(hook_id),
            _ => {
                info!("no known event by the name {}.", hook.event);
                return Err(());
            }
        }
        self.hooks.insert(hook_id, hook);

        self.next_uid += 1;
        Ok(())
    }

    fn update(&mut self, new_db: HookDB) {
        for (_, hook) in new_db.hooks.iter() {
            if self.add_hook(hook.clone()).is_err() {
                error!("unknown error merging in new hook");
            };
        }
    }
}

// #[derive(Clone)]
// pub struct Hook {
//     pub event: String,
//     pub exec: String,
//     // uid: Option<u16>,  // gets set by the event checker
// }

#[derive(Clone)]
pub struct HookData {
    pub send: Sender<HookDB>,
    pub cmd: Sender<msgs::EventCmd>,
    pub db: HookDB,
}

/// sends the new hook db to the event loop.
async fn update_hook_db(hook_db: &HookDB, event_loop_tx: &Sender<HookDB>) -> u8 {
    match event_loop_tx.send(hook_db.clone()).await {
        Ok(_) => 0,
        Err(_) => 8,
    }
}

async fn make_db_from_conf(hooks: Vec<Hook>, db: &mut HookDB) {
    for conf_hook in hooks {
        match db.add_hook(conf_hook.clone()) {
            Ok(_) => {}
            Err(_) => error!(
                "could not add a hook from the config file. bad hook: {:?}",
                conf_hook
            ),
        };
    }
}

/// adds a hook by sending a mpsc message to start hooks.?
async fn add_hook(args: &str, hook_data: &mut HookData) -> u8 {
    let (event, exec) = match args.split_once(' ') {
        Some((ev, ex)) => (ev, ex),
        None => return 7,
    };

    let hook = Hook {
        event: event.to_string(),
        exec: exec.to_string(),
    };
    match hook_data.db.add_hook(hook) {
        Ok(_) => update_hook_db(&hook_data.db, &hook_data.send).await,
        Err(_) => 9,
    }
}

/// removes a hook by sending a mpsc message to start hooks.
async fn rm_hook(_args: &str, _hook_db: &mut HookDB) -> u8 {
    // TODO: Implement
    0
}

/// used to get a list of hooks from start hooks.
/// returns a string of a nice table representing all hooks,
/// what their hooked to and their ID.
async fn get_hook(_hook_db: &HookDB) -> String {
    // TODO: Implement
    String::new()
}

fn execute_hook(exec: &str, context: HashMap<String, String>) {
    let exec = String::from(exec);

    tokio::task::spawn(async move {
        let proc = Command::new("sh")
            .arg("-c")
            .arg(&exec) // format!("systemd-run --user {exec}")  // adding systemd-run doesnt work.
            // .env_clear()
            .envs(context)
            .spawn(); // maybe change to .output() or remove .wait().

        match proc {
            Ok(mut child) => {
                if let Err(reason) = child.wait().await {
                    error!("hook: '{exec}', exited with error: \"{reason}\"");
                } else {
                    debug!("child id => {child:?}");
                }
            }
            Err(reason) => {
                error!("running hook: '{exec}' produced error: {reason}. this usually means the executable wasn't found in the PATH.");
            }
        }
    });
}

fn run_hooks(context: Option<HashMap<String, String>>, event_hook: &[HookID], all_hooks: &Hooks) {
    let context = match context {
        Some(con) => con,
        None => return, // might need to change this return to an empty hashmap if any events don't return a context.
    };

    let hooks = get_hooks(event_hook, all_hooks);

    for hook in hooks {
        execute_hook(&hook.exec, context.clone())
    }
}

fn get_hooks(event_hooks: &[HookID], all_hooks: &Hooks) -> Vec<Hook> {
    let mut hooks = Vec::new();

    for hook_id in event_hooks {
        match all_hooks.get(hook_id) {
            Some(prog) => hooks.push(prog.clone()),
            None => continue,
        }
    }

    hooks
}

pub async fn hooks_switch(
    cmd: &str,
    args: &str,
    maybe_hook_data: &mut Option<HookData>,
) -> OptGenRes {
    match (cmd, maybe_hook_data) {
        ("add-hook", Some(hook_data)) => Some((add_hook(args, hook_data).await, None)),
        ("rm-hook", Some(hook_data)) => Some((rm_hook(args, &mut hook_data.db).await, None)),
        ("ls-hook" | "list-hook", Some(hook_data)) => {
            // TODO: this chould be a table, just like sql output. Thats why its called table.
            let table = get_hook(&hook_data.db).await;
            Some((0, Some(table)))
        }
        _ => None,
    }
}

/// starts asynchronously checking for events and then triggers hooks.
pub async fn check_even_hooks(
    hook_db_rx: &mut Receiver<HookDB>,
    cmd_passer: &mut Receiver<msgs::EventCmd>,
    stop_execs: HashSet<String>,
    config_hooks: Vec<Hook>,
    ignore_web: bool,
) {
    // define the hook storage struct
    let mut hook_db = HookDB::new();
    make_db_from_conf(config_hooks, &mut hook_db).await;

    // port change event
    let (ports_tx, mut ports_rx) = unbounded_channel::<Vec<Context>>();
    let ports = task::spawn(events::port_change(stop_execs, ignore_web, ports_tx));

    // bluetooth device connected event
    let (blt_tx, mut blt_rx) = unbounded_channel::<Context>();
    let bluetooth_dev = task::spawn(events::blt_dev_conn(blt_tx));

    // new usb dev event
    let (usb_tx, mut usb_rx) = unbounded_channel::<Context>();
    let new_usb = task::spawn(events::new_usb(usb_tx));

    // change in backlight event
    let (bl_tx, mut bl_rx) = unbounded_channel::<Context>();
    let backlight = task::spawn(events::backlight_change(bl_tx));

    // network (dis)connected event
    let (net_con_tx, mut net_con_rx) = unbounded_channel::<Context>();
    let net_connected = task::spawn(events::network_connection(net_con_tx));

    // changed wifi network event.
    let (wifi_net_tx, mut wifi_net_rx) = unbounded_channel::<Context>();
    let network_change = task::spawn(events::wifi_change(wifi_net_tx));

    loop {
        // async check for events and messages via thread based message passing
        tokio::select! {
            Some(cmd) = cmd_passer.recv() => {
                match cmd {
                    msgs::EventCmd::Exit => break,
                }
            }
            message = hook_db_rx.recv() => {
                match message {
                    Some(tmp_hook_db) => hook_db.update(tmp_hook_db),
                    None => {
                        error!("failed to receive the modified hook database.");
                    }
                }
            },
            context = wifi_net_rx.recv() => {
                info!("running event hooks for event 'wifi-network-change'");
                run_hooks(context, &hook_db.wifi_net, &hook_db.hooks);
            },
            context = net_con_rx.recv() => {
                info!("running event hooks for event 'network-toggle'");
                run_hooks(context, &hook_db.net_con, &hook_db.hooks);
            },
            // context = events::file_exists() => run_hooks(context, &hook_db.test_file_exists, &hook_db.hooks).await,
            context = bl_rx.recv() => {
                info!("running event hooks for event 'backlight change'");
                run_hooks(context, &hook_db.backlight, &hook_db.hooks);
            },
            context = usb_rx.recv() => {
                info!("running event hooks for event 'new-usb-device'");
                run_hooks(context, &hook_db.usb_dev, &hook_db.hooks);
            },
            context = blt_rx.recv() => {
                info!("running event hooks for event 'bluetooth-device'");
                run_hooks(context, &hook_db.bluetooth_conn, &hook_db.hooks);
            },
            contexts = ports_rx.recv() => {
                info!("running event hooks for event 'port-status-change'");

                if let Some(contexts) = contexts {
                    for context in contexts {
                        run_hooks(Some(context), &hook_db.ports_change, &hook_db.hooks);
                    }
                }
            }
        }
    }

    for event_listener in [
        ports,
        bluetooth_dev,
        new_usb,
        backlight,
        net_connected,
        network_change,
    ] {
        event_listener.abort();
        info!("stopping event listener: {:?}", event_listener);
    }
}

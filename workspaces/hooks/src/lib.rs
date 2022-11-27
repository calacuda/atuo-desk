use tokio::sync::mpsc::{Sender, Receiver};
use tokio::process::Command;
use std::collections::{HashMap, HashSet};
use futures_util::future::join_all;
use config::Hook;
use config::OptGenRes;

pub type HookID = u16;
pub type Hooks = HashMap<HookID, Hook>;

#[derive(Clone)]
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
        let event = hook.event.clone();
        self.hooks.insert(hook_id, hook);
        match event.as_str() {
            "wifi-network-change" => self.wifi_net.push(hook_id),
            "network-toggle" => self.net_con.push(hook_id),
            "backlight" => self.backlight.push(hook_id),
            "new-usb-device" => self.usb_dev.push(hook_id),
            "bluetooth-device" => self.bluetooth_conn.push(hook_id),
            "port-status-change" => self.ports_change.push(hook_id),
            // "test_file_exists" => self.test_file_exists.push(hook_id),
            _ => {
                println!("[INFO] no known event by the name {event}.");
                return Err(())
            }
        }
        self.next_uid += 1;
        Ok(())
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
            Ok(_) => {},
            Err(_) => println!("could not add a hook from the config file. bad hook: {:?}", conf_hook),
        };
    }
}

/// adds a hook by sending a mpsc message to start hooks.?
async fn add_hook(args: &str, hook_data: &mut HookData) -> u8 {
    let (event, exec) = match args.split_once(' ') {
        Some((ev, ex)) => (ev, ex),
        None => return 7,
    };

    let hook = Hook { event: event.to_string(), exec: exec.to_string() };
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

async fn run_hooks(context: HashMap<String, String>, event_hook: &[HookID], all_hooks: &Hooks) {
    let hooks = get_hooks(event_hook, all_hooks).await;
    for hook in hooks {
        let _ = Command::new("sh")
        .arg("-c")
        .arg(&hook.exec)
        // .env_clear()
        .envs(&context)
        .spawn()
        // .expect(&format!("could not run hook: '{}'", hook.exec))
        .unwrap_or_else(|e| 
            { 
                println!("[ERROR] could not run hook: '{}'\ngot error:\n{}", hook.exec, e);
                panic!("") 
            }
        )
        .wait()
        .await;
    }
} 

async fn get_hooks(event_hooks: &[HookID], all_hooks: &Hooks) -> Vec<Hook> {
    let mut hooks = Vec::new();

    for hook_id in event_hooks { 
        match all_hooks.get(hook_id) {
            Some(prog) => hooks.push(prog.clone()),
            None => continue,
        }
    }

    hooks
}

/// starts asynchronously checking for events and then triggers hooks.
pub async fn check_even_hooks(hook_db_rx: &mut Receiver<HookDB>, stop_execs: HashSet<String>, config_hooks: Vec<Hook>) {
    // define the hook storage struct
    let mut hook_db = HookDB::new();
    // stops bluetooth from registering devices twice.
    let mut conn_bt_dev = HashSet::new();
    
    make_db_from_conf(config_hooks, &mut hook_db).await;
    // TODO: use the same boxed future technique from the switch board function from server/main.rs then use Join_all!()
    loop {
        // async check for events and messages via thread based message passing
        tokio::select! {
            message = hook_db_rx.recv() => {
                match message {
                    Some(tmp_hook_db) => hook_db = tmp_hook_db,
                    None => {
                        println!("[ERROR] failed to receive the modified hook database.");
                    }
                }
            },
            context = events::wifi_change() => run_hooks(context, &hook_db.wifi_net, &hook_db.hooks).await,
            context = events::network_connection() => run_hooks(context, &hook_db.net_con, &hook_db.hooks).await,
            // context = events::file_exists() => run_hooks(context, &hook_db.test_file_exists, &hook_db.hooks).await,
            context = events::backlight_change() => run_hooks(context, &hook_db.backlight, &hook_db.hooks).await,
            context = events::new_usb() => run_hooks(context, &hook_db.usb_dev, &hook_db.hooks).await,
            // context = events::discovered_blt() => run_hooks(context, &hook_db.bluetooth_dev, &hook_db.hooks).await,
            context = events::blt_dev_conn(&mut conn_bt_dev) => run_hooks(context, &hook_db.bluetooth_conn, &hook_db.hooks).await,
            contexts = events::port_change(&stop_execs) => {
                let mut hooks = Vec::new(); 
                for context in contexts {
                    // println!("{:?}", context);
                    hooks.push(run_hooks(context, &hook_db.ports_change, &hook_db.hooks));
                }
                let _ = join_all(hooks).await;
            }
        }
    }
}

pub async fn hooks_switch( 
    cmd: &str, 
    args: &str, 
    maybe_hook_data: &mut Option<HookData>,
) -> OptGenRes {
    match (cmd, maybe_hook_data, ) { 
        ( "add-hook", Some(hook_data) )=> Some((add_hook(args, hook_data).await, None)),
        ( "rm-hook", Some(hook_data) )=> Some((rm_hook(args, &mut hook_data.db).await, None)),
        ( "ls-hook" | "list-hook", Some(hook_data) )=> {
            // TODO: this chould be a table, just like sql output. Thats why its called table.
            let table = get_hook(&hook_data.db).await;
            Some((0, Some(table)))
        },
        _ => None,
    }
}
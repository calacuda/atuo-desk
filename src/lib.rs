pub mod bspwm;
pub mod common;
pub mod config;
// pub mod events;
// pub mod hooks;
pub mod client;
pub mod leftwm;
pub mod msgs;
pub mod qtile;
pub mod server;
pub mod wm_lib;

pub const MSG_ERROR: char = 7 as char;
pub const MSG_SUCCESS: char = 0 as char;
pub const MSG_DELIM: char = 1 as char;


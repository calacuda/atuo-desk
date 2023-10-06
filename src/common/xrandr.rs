use log::error;
use std::process::{Command, Stdio};

pub fn add_monitor(monitor: &str) -> u8 {
    return match Command::new("xrandr")
        .args(["--output", monitor, "--auto"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(_) => 0,
        Err(e) => {
            error!("couldn't not add monitor via xrandr: {}", e);
            4
        }
    };
}

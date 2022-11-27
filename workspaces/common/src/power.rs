use std::process::Command;

/*
 * TODOs:
 *
 * add configurability for power commands.
 *
 */

pub fn power_off() -> u8 {
    let _process = Command::new("systemctl").args(["poweroff"]).output();
    0
}

pub fn hibernate() -> u8 {
    let _process = Command::new("systemctl").args(["hibernate"]).output();
    0
}

pub fn reboot() -> u8 {
    let _process = Command::new("systemctl").args(["reboot"]).output();
    0
}

pub fn sleep() -> u8 {
    let _process = Command::new("systemctl")
        .args(["suspend-then-hibernate"])
        .output();
    0
}

pub fn lock() -> u8 {
    let _process = Command::new("loginctl").args(["lock-session"]).output();
    0
}

pub fn logout() -> u8 {
    let _process = Command::new("pkill").args(["bspwm"]).output();
    0
}

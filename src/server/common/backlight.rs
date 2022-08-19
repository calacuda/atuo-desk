use std::process::Command;

/*
 * TODOs:
 * - add programatic alsa control instead of using shell command.
 */

fn xbacklight(dir: &str, amount: &str) -> u8 {
    return match Command::new("xbacklight").args([dir, amount]).output() {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] xbacklight {}, {} error: {}", dir, amount, e);
            4
        }
    };
}

pub fn inc_bright(amount: &str) -> u8 {
    return xbacklight("-inc", amount);
}

pub fn dec_bright(amount: &str) -> u8 {
    return xbacklight("-dec", amount);
}

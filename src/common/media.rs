use std::process::Command;

/*
 * TODOs:
 * - add programmatic ALSA control instead of using shell command.
 */

fn change_volume(amount: &str, raise: bool) -> u8 {
    // changes volume by amount. use positive ints to add volume,
    // and negative to lower volume.
    let process = Command::new("amixer")
        .args([
            "set",
            "Master",
            &format!("{}%{}", amount, if raise { "+" } else { "-" }),
        ])
        .output();
    match process {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] ALSA volume error: {}", e);
            4
        }
    }
}

pub fn volume_up(amount: &str) -> u8 {
    // raises volume by amount
    change_volume(amount, true)
}

pub fn volume_down(amount: &str) -> u8 {
    // lowers the volume by amount
    change_volume(amount, false)
}

pub fn mute() -> u8 {
    match Command::new("amixer")
        .args(["-D", "pulse", "set", "Master", "1+", "toggle"])
        .output()
    {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] alsa volume error: {}", e);
            4
        }
    }
}

fn playerctl(arg: &str) -> u8 {
    match Command::new("playerctl").args([arg]).output() {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] playerctl {} error: {}", arg, e);
            4
        }
    }
}

pub fn play_pause() -> u8 {
    playerctl("play-pause")
}

pub fn play() -> u8 {
    playerctl("play")
}

pub fn pause() -> u8 {
    playerctl("pause")
}

pub fn stop() -> u8 {
    playerctl("stop")
}

pub fn next_track() -> u8 {
    playerctl("next")
}

pub fn last_track() -> u8 {
    playerctl("previous")
}

use std::process::Command;

/*
 * TODOs:
 * - add programatic alsa control instead of using shell command.
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
    return match process {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] alsa volume error: {}", e);
            4
        }
    };
}

pub fn volume_up(amount: &str) -> u8 {
    // raises volume by amount
    return change_volume(amount, true);
}

pub fn volume_down(amount: &str) -> u8 {
    // lowers the volume by amount
    return change_volume(amount, false);
}

pub fn mute() -> u8 {
    return match Command::new("amixer")
        .args(["-D", "pulse", "set", "Master", "1+", "toggle"])
        .output()
    {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] alsa volume error: {}", e);
            4
        }
    };
}

fn playerctl(arg: &str) -> u8 {
    return match Command::new("playerctl").args([arg]).output() {
        Ok(_) => 0,
        Err(e) => {
            println!("[ERROR] playerctl {} error: {}", arg, e);
            4
        }
    };
}

pub fn play_pause() -> u8 {
    return playerctl("play-pause");
}

pub fn play() -> u8 {
    return playerctl("play");
}

pub fn pause() -> u8 {
    return playerctl("pause");
}

pub fn stop() -> u8 {
    return playerctl("stop");
}

pub fn next_track() -> u8 {
    return playerctl("next");
}

pub fn last_track() -> u8 {
    return playerctl("previous");
}

// use std::collections::HashMap;
// use std::error::Error;
// use std::io::prelude::*;
use std::process::Command;
// use std::thread;

pub mod media;
pub mod power;

pub fn open_program(program: &str) -> u8 {
    println!("[LOG] running: {}", program);
    let mut process = Command::new(program)
        .output()
        .expect("failed to execute process");
    println!("[LOG] ran {}", program);
    return 0;
}

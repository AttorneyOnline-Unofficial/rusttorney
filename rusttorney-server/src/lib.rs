#![forbid(unsafe_code)]
#![allow(unused)]

use std::io::{stdin, BufRead, Read};

pub mod client_manager;
pub mod command;
pub mod config;
pub mod master_server_client;
pub mod music_list;
pub mod networking;
pub mod server;

fn prompt(text: &str) -> bool {
    let mut answer = String::with_capacity(3);
    let mut stdin = stdin();

    loop {
        log::warn!("{} [Y/n]", text);
        answer.clear();
        stdin.lock().read_line(&mut answer);

        match answer.trim() {
            "y" | "yes" | "Y" => return true,
            "n" | "no" | "N" => return false,
            _ => continue,
        }
    }
}

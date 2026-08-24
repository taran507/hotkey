// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::fmt;

fn main() {
    fmt()
        .json()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    println!("Hello, world!");
    hotkey_lib::run()
}

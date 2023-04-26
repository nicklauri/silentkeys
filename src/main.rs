#![allow(warnings)]
use std::thread;

mod buffer;
mod config;
mod input;
mod noti;
mod sys;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const PROCESS_NAME: &'static str = concat!(env!("CARGO_PKG_NAME"), ".exe");

/**
 * TODO:
 * - log to file.
 * - config file.
 * - arguments.
 * - fix tray item.
 */

fn main() -> anyhow::Result<()> {
    println!(
        "SilentKeys version {VERSION}\n\
    Copyright by Nick Lauri (c) 2023\n\
    This is chattering keys ANNIHILATION!!!"
    );

    println!("info: listen for events");

    noti::app_is_running();

    // input::handle_key_chattering_events();
    input::handle_key_homemade();

    Ok(())
}

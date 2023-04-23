#![allow(warnings)]
use std::thread;

use crate::system_tray::SystemTrayMessage;

mod input;
mod system_tray;
mod buffer;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/**
 * TODO:
 * - log to file.
 * - config file.
 * - arguments.
 * - fix tray item.
 */

fn main() -> anyhow::Result<()> {
    println!("SilentKeys version {VERSION}\n\
    Copyright by Nick Lauri (c) 2023\n\
    This is chattering keys ANNIHILATION!!!");

    let rx = system_tray::init_system_tray()?;

    println!("info: listen for events");

    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(SystemTrayMessage::Quit) => {
                    println!("info: quit message received, cleaning up and exitting...");
                    std::process::exit(0);
                },
                _ => {}
            }
        }
    });

    input::handle_key_chattering_events();

    Ok(())
}
